run *args:
	cargo run {{args}}

format:
	cargo +nightly fmt

lint:
	cargo clippy --all

test:
	cargo test --all
