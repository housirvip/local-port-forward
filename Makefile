.PHONY: dev dev-frontend dev-backend build frontend backend clean

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
