use serde_json::Value;

// Deep JSON merge patch (RFC 7386)
pub fn merge_patch(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(a_obj), Value::Object(b_obj)) => {
            for (k, v) in b_obj {
                if v.is_null() {
                    a_obj.remove(k);
                } else if let Some(target) = a_obj.get_mut(k) {
                    merge_patch(target, v);
                } else {
                    a_obj.insert(k.clone(), v.clone());
                }
            }
        }
        (Value::Array(a_arr), Value::Object(b_obj))
            if b_obj.keys().all(|k| k.parse::<usize>().is_ok()) && !b_obj.is_empty() =>
        {
            // The F1 API deviates from RFC 7386 by sending delta updates for arrays as objects
            // where the keys are string numerical indices (e.g. "Sectors": {"0": {"Value": "..."}}).
            // This ensures we update the specific indices in our array instead of replacing it.
            for (k, v) in b_obj {
                if let Ok(idx) = k.parse::<usize>() {
                    if idx >= a_arr.len() {
                        a_arr.resize(idx + 1, Value::Null);
                    }
                    merge_patch(&mut a_arr[idx], v);
                }
            }
        }
        (a_val, b_val) => {
            a_val.clone_from(b_val);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_merge_patch_basic() {
        let mut a = json!({
            "existing": "old",
            "nested": { "child": "value", "child2": "value2"},
            "array": [1, 2],
            "to_remove": "bye"
        });

        let b = json!({
            "existing": "new",
            "new_key": 42,
            "nested": { "child": "new", "child3": "value3" },
            "array": [3, 4],
            "to_remove": null
        });

        merge_patch(&mut a, &b);

        assert_eq!(
            a,
            json!({
                "existing": "new",
                "new_key": 42,
                "nested": { "child": "new", "child2": "value2" , "child3": "value3"},
                "array": [3, 4]
            })
        );
    }

    #[test]
    fn test_merge_patch_array_numeric_keys() {
        // The F1 SignalR API sends some initial payloads as arrays (like Sectors or Stints)
        // but sends updates as JSON objects, where the object keys are the stringified
        // numerical indices.
        // We must ensure the merge_patch properly maps these object keys back into the
        // existing JSON array indices rather than overwriting the entire array.
        let mut a = json!([
            {"Value": "1"},
            {"Value": "2"}
        ]);

        let b = json!({
            "1": {"Value": "3"},
            "2": {"Value": "4"}
        });

        merge_patch(&mut a, &b);

        assert_eq!(
            a,
            json!([
                {"Value": "1"},
                {"Value": "3"},
                {"Value": "4"}
            ])
        );
    }
}
