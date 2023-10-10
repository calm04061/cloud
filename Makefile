build:
	cargo build
build-image:
	docker buildx build --platform linux/arm64/v8,linux/amd64 -t ghcr.io/calm04061/cloud:develop .