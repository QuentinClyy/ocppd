# ocppd

A sans-IO OCPP charging-station simulator. It runs station state machines
that a CSMS can connect to, driven by JSON-line commands on stdin, with
events as JSON lines on stdout. OCPP 1.6J first, 2.0.1 later.

## Build

```
cargo build --release
```

## Test

```
cargo fmt --check
cargo test
```

## Run

Commands go in on stdin, one JSON object per line. Events come out on stdout,
one JSON object per line. Diagnostics go to stderr.

```
echo '{"command":"spawn","prefix":"CP","count":1}' |
  cargo run --release -- --csms ws://localhost:8180/steve/websocket/CentralSystemService
```

This starts station `CP1`, which connects to `<csms>/CP1` using the
`ocpp1.6` subprotocol.
