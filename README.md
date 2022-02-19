# bot

This is a bot program for the essayshark website that listens for new orders and bids whenever the writer's allocated time for a bid elapses.
It is designed to bypass Cloudflare DDoS protection, autocompete with other bots on the site and also simulate file downloads for orders with attachments.

## Prerequisites

### Install [node](https://nodejs.org/en/download/)

Confirm installation by running:
```
node -v
```
which should output the version of node installed.

### Install [Rust](https://www.rust-lang.org/tools/install)

Confirm the installation by running:
```
cargo -V
```
which should also output the version of rust installed.

## How To Run

The project requires that the user provide their writer account username and password for the essayshark site. 
This can be configured in the bypass.ts file present within the '/src' project's directory.

The project's setup is bundled into a `Makefile`. Running:
```
make
```
starts the bot.

The cloudflare bypasser seems not to work on Google Cloud but trying it out in AWS does the trick. 