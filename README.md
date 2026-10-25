# ocppd

Simulate OCPP charging stations against your CSMS: one station or ten thousand,
OCPP 1.6J or 2.0.1, driven from a terminal or a script.

- **Real protocol, simulated hardware.** Stations boot, heartbeat, authorize,
  charge with a realistic curve, meter, reserve, update firmware and handle
  certificates. EVs, cables, faults and power cuts are commands you send.
- **Scriptable.** Commands are JSON lines on stdin; everything that happens
  comes back as JSON lines on stdout.
- **Light.** A thousand stations take about 30 MB.

## Install

Download the binary for your system from the
[latest release](https://github.com/QuentinClyy/ocppd/releases/latest). It is
a single file with no dependencies:

```
curl -L https://github.com/QuentinClyy/ocppd/releases/latest/download/ocppd-x86_64-unknown-linux-musl.tar.gz | tar xz
sudo mv ocppd-x86_64-unknown-linux-musl/ocppd /usr/local/bin/
```

| System | Archive |
|---|---|
| Linux x86_64 | `ocppd-x86_64-unknown-linux-musl.tar.gz` |
| Linux arm64 | `ocppd-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple silicon | `ocppd-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `ocppd-x86_64-apple-darwin.tar.gz` |

Each archive has a `.sha256` next to it, and a build attestation you can check
with `gh attestation verify <archive> -R QuentinClyy/ocppd`.

The macOS binary is not signed. If you downloaded it with a browser, clear the
quarantine flag with `xattr -d com.apple.quarantine ocppd`.

To build from source, with Rust installed:

```
cargo install --path ocppd --features vendored
```

`--features vendored` builds OpenSSL in; without it, ocppd links the system
OpenSSL and needs its headers (`apt install libssl-dev pkg-config`).

## Quick start

Point ocppd at a CSMS. Every station connects to `<url>/<station id>`:

```
ocppd --csms ws://localhost:8180/steve/websocket/CentralSystemService
```

Then type commands, one per line:

```
{"command":"spawn","prefix":"CP","count":1}
{"command":"plug_in","station":"CP1","connector":1}
{"command":"authorize","station":"CP1","id_tag":"TAG1"}
{"command":"stop","station":"CP1","connector":1}
```

`CP1` boots, starts a transaction once the CSMS accepts `TAG1`, sends meter
values every minute, and stops. Each step prints an event:

```
{"station":"CP1","at":1792893141250,"kind":"boot_status","status":"Accepted"}
{"station":"CP1","at":1792893149252,"kind":"transaction_started","connector":1,"transaction_id":42}
```

For OCPP 2.0.1, spawn with `"ocpp":"2.0.1"`:

```
{"command":"spawn","prefix":"CS","count":1,"ocpp":"2.0.1"}
```

## Documentation

- [Usage](docs/usage.md): the command line, every stdin command, and the events.
- [Configuration](docs/configuration.md): station settings, security profiles and TLS.
- [Simulation](docs/simulation.md): what is simulated, how, and at what scale.
- [OCPP 1.6](docs/ocpp-1.6.md) and [OCPP 2.0.1](docs/ocpp-2.0.1.md): what is supported.
- [ocppd.opencli.json](docs/ocppd.opencli.json): the CLI described in the
  [OpenCLI](https://opencli.org) format.

## Development

```
cargo fmt --check
cargo clippy --all-targets
cargo test
```

Unit tests sit next to the code they test, as `<file>.test.rs`. Test fixtures
(a fake CSMS, a test certificate authority) live in `ocppd-core/tests/fixtures`.
The only integration tests, in `ocppd/tests`, run the binary.

## License

Apache-2.0
