release:
	@echo "Building..."
	cargo build --release
	
debug:
	@echo "Debug mode enabled."
	cargo run -- --debug

docker:
	@echo "Building for docker."
	docker build . -t pmanager

install:
	cargo install --path .
