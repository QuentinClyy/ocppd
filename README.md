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

```
cargo run --release
```
