.PHONY: dev dev-frontend dev-backend build frontend backend clean desktop-dev desktop-build

frontend:
	cd frontend && npm install && npm run build

backend:
	cargo build --release

build: frontend backend

dev:
	@trap 'kill 0' INT TERM; \
	(cd frontend && npm run dev -- --host --port 5173) & \
	(RUST_LOG=debug DB_PATH=dev.db cargo run) & \
	wait

dev-frontend:
	cd frontend && npm run dev -- --host --port 5173

dev-backend:
	RUST_LOG=debug DB_PATH=dev.db cargo run

clean:
	rm -rf web dev.db
	cargo clean

# Desktop app targets (requires: cargo install tauri-cli --version '^2' --locked)
# Linux also requires: sudo apt-get install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf build-essential libssl-dev
desktop-dev:
	cargo tauri dev

desktop-build:
	cargo tauri build
