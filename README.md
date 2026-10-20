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

### Commands

| Command | Effect |
|---|---|
| `{"command":"spawn","prefix":"CP","count":2}` | start stations `CP1` and `CP2` |
| `{"command":"spawn","prefix":"CP","count":100,"jitter":false}` | start an aligned fleet: every station heartbeats and reconnects at the same instant (load storms) |
| `{"command":"plug_in","station":"CP1","connector":1}` | plug an EV into connector 1 |
| `{"command":"authorize","station":"CP1","id_tag":"TAG1"}` | present a card; starts a transaction once accepted and plugged in (an optional `"connector":1` picks the connector) |
| `{"command":"stop","station":"CP1","connector":1}` | stop the transaction locally |
| `{"command":"unplug","station":"CP1","connector":1}` | unplug the EV; stops a running transaction |

`plug_in` takes an optional EV, with these defaults:
`"ev":{"capacity_kwh":60,"soc":0.2,"max_kw":11,"target_soc":1.0}`.
The station draws at most 230 V × 32 A × 3 phases (22 kW). Meter values
follow a charging curve that is flat up to 80 % and tapers after that.
