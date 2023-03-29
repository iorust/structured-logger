# options
ignore_output = &> /dev/null

.PHONY: doc test lint

doc:
	@cargo doc --open

lint:
	@cargo fmt
	@cargo clippy --all-targets --all-features

test:
	@cargo fmt
	@cargo clippy --all-targets --all-features
	@cargo test -- --nocapture --test-threads=1
