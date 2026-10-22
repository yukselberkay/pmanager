release:
	@echo "Building..."
	cargo build --release
	
docker:
	@echo "Building for docker."
	docker build . -t pmanager

install:
	cargo install --path .
