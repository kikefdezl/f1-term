run *args:
	cargo run {{args}}

format:
	cargo +nightly fmt

lint:
	cargo clippy

test:
	cargo test --all
