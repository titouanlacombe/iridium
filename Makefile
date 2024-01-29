COMPOSE=docker compose -p iridium

run:
	RUST_BACKTRACE=full RUST_LOG=info cargo run --release
