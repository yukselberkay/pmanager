release:
	make release-linux

release-linux:
	@echo "Building for linux.."
	cargo build --release
	
debug:
	cargo run -- debug
