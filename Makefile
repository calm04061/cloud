build:
	cargo build
build-base:
	docker build -t cloud:aarch64-build --progress=plain -f support/Dockerfile .
build-image:
	cargo clean
	docker run -it  --name cloud-builder -v ./.cargo:/src/.cargo -v ./lib:/src/lib -v ./Cargo.toml:/src/Cargo.toml cloud:aarch64-build cargo build --release
	mkdir -p target/release
	docker cp cloud-builder:/src/target/release/cloud target/release/
	docker cp cloud-builder:/src/target/release/libcloud_ui.so target/release
	docker rm cloud-builder
deploy-pi: build-image
	ssh pi sudo systemctl stop cloud
	scp target/release/cloud pi:/opt/cloud/
	scp target/release/lib*.so pi:/opt/cloud/plugin/
	ssh pi sudo systemctl start cloud
