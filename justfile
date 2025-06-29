set fallback
clippy:
	cargo clippy -- -W clippy::pedantic

release:
	cargo build --release
