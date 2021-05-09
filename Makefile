.PHONY: it doc

it:
	cargo fmt
	cargo clippy
	cargo doc --no-deps
	cargo test
	cargo run

doc:
	cargo doc --no-deps
