build:
	cargo build
build-ui:
	cd lib/cloud-ui/web && npm install && npm run build
build-base:
	docker build -t cloud:aarch64-build --progress=plain -f support/Dockerfile .
build-image:
	cargo clean
	docker run -it  --name cloud-builder -v ./.cargo:/src/.cargo -v ./lib:/src/lib -v ./Cargo.toml:/src/Cargo.toml -v ./Cargo.lock:/src/Cargo.lock \
  	   -e "GOOGLE_DRIVE_APP_SECRET=123" \
  	   -e "GOOGLE_DRIVE_APP_ID=123" cloud:aarch64-build cargo build --release
	mkdir -p target/release
	docker cp cloud-builder:/src/target/release/cloud target/release/
	docker cp cloud-builder:/src/target/release/libcloud_ui.so target/release
	docker rm cloud-builder
deploy-pi: build-image
	ssh pi sudo systemctl stop cloud
	scp target/release/cloud pi:/opt/cloud/
	scp target/release/lib*.so pi:/opt/cloud/plugin/
	ssh pi sudo systemctl start cloud
