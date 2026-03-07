# f1-term-signalr

This crate handles the network connection to the Formula 1 SignalR WebSocket API, parsing the raw JSON payloads, and converting them into clean core domain structs for `f1-term-core`.

## Architecture & Data Flow

This crate implements a strict Anti-Corruption Layer (ACL) to fully decouple F1's raw JSON parsing from our core domain model. The data pipeline works as follows:

### 1. Network & State Loop (`client.rs`)

1. After successful connection, `SignalRF1Client` reads a JSON string from the WebSocket.
2. It parses the JSON and looks for the `"R"` (initial state) or `"M"` (partial patch) payloads.
3. It applies any partial patches using `merge_patch` onto its `self.canonical_state` (`serde_json::Value`).

### 2. The Extraction & Conversion Pipeline (`extract.rs` -> `parsing/` -> `convert/`)

4. While doing the previous step, it keeps track of a `Vec<Topic>` representing exactly which F1 topics were modified in the last tick (`updated_topics`).
5. `client.rs` calls `extract_updates(&self.canonical_state, &updated_topics)`, which attempts to build a unified `TelemetryUpdate` frame. To do this, it calls an isolated function for **each** core domain type (e.g., `extract_timing_data_update`).
7. Inside each isolated function:
   - **Filter:** It first checks: _"Were any of the topics that I care about updated in this frame?"_ (e.g. `updated_topics.contains(&Topic::TimingData)`). If no, it immediately returns `None`.
   - **Extract:** If yes, it pulls the relevant JSON data out of the `canonical_state`.
   - **Parse (Raw):** It passes that JSON to a specific parsing function in the `parsing/` module. This strictly validates that the JSON matches the F1 API's schema and deserializes it into `Raw` structs.
   - **Convert (Core):** It passes the `Raw` structs into a specific conversion function in the `convert/` module. This transforms the raw API data into our clean, core domain types.
   - **Return:** It returns the parsed domain type wrapped in an `Option<T>`
8. The constructed `TelemetryUpdate` frame is returned by the client back to the `TelemetryEngine`, which then applies those `Some(T)` patches to the core `TelemetryState`.
