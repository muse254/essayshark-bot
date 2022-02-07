.PHONY: setup
setup: installs get-cookie run-bot

.PHONY: run-bot
run-bot:
	cargo run --release

.PHONY: get-cookie
get-cookie:
	cd src && npm test

installs:
	cargo build --release && cd src && npm i

clean:
	cargo clean && cd src && rm -rf node_modules