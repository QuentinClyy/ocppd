# ocppd

Simulate OCPP charging stations against your CSMS: one station or ten thousand,
OCPP 1.6J or 2.0.1, driven from a terminal or a script.

- **Real protocol, simulated hardware.** Stations boot, heartbeat, authorize,
  charge with a realistic curve, meter, reserve, update firmware and handle
  certificates. EVs, cables, faults and power cuts are commands you send.
- **Interactive or scriptable.** In a terminal, a TUI shows the fleet and drives
  it with a few keys. In a script, commands are JSON lines on stdin and
  everything that happens comes back as JSON lines on stdout.
- **Light.** A thousand stations take about 30 MB.

## Install

On Linux or macOS (x86_64 or arm64):

```
curl -fsSL https://raw.githubusercontent.com/QuentinClyy/ocppd/main/install.sh | sh
```

It downloads the [latest release](https://github.com/QuentinClyy/ocppd/releases/latest),
checks its checksum, and puts `ocppd` and `ocppd-tui`, the terminal UI, in
`~/.local/bin`. Run it again to update. To read it before running it:

```
curl -fsSLO https://raw.githubusercontent.com/QuentinClyy/ocppd/main/install.sh
less install.sh && sh install.sh
```

| Variable | Default | Meaning |
|---|---|---|
| `OCPPD_INSTALL_DIR` | `~/.local/bin` | where the two binaries go |
| `OCPPD_DOWNLOAD` | the latest release | where to download from, e.g. `https://github.com/QuentinClyy/ocppd/releases/download/v0.3.0` for a given version |

Every archive also has a build attestation:
`gh attestation verify <archive> -R QuentinClyy/ocppd`.

To build from source, with Rust and Go installed:

```
cargo install --path ocppd --features vendored
(cd ocppd-tui && go build -o ~/.cargo/bin/ocppd-tui .)
```

`--features vendored` builds OpenSSL in; without it, ocppd links the system
OpenSSL and needs its headers (`apt install libssl-dev pkg-config`).

## Quick start

Point ocppd at a CSMS. Every station connects to `<url>/<station id>`:

```
ocppd --csms ws://localhost:8180/steve/websocket/CentralSystemService
```

The [terminal UI](docs/tui.md) opens:

1. `n` opens the spawn form: `tab`, type an id such as `CP1`, then `enter`.
2. `enter` opens the station, `j` selects its connector.
3. `p` plugs an EV in, `a` asks for an id tag: type `TAG1`, then `enter`.
4. The connector shows `Charging`, with its power and state of charge. `s`
   stops, `?` lists every key.
5. `q` quits and leaves the stations running in a background
   [daemon](docs/daemon.md); `ocppd` again comes back to them, `Q` stops them.

## Docker

```
docker run -d -p 127.0.0.1:7600:7600 -e OCPPD_TOKEN=s3cret -e OCPPD_CSMS=ws://csms.example/ocpp ghcr.io/quentinclyy/ocppd
OCPPD_TOKEN=s3cret ocppd-tui --connect tcp:127.0.0.1:7600
```

## Scripting

With `--json`, or from a pipe, ocppd reads commands as JSON lines:

```
ocppd --json --csms ws://localhost:8180/steve/websocket/CentralSystemService
```

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

- [Terminal UI](docs/tui.md): the screen, the keys and the spawn form.
- [Daemon](docs/daemon.md): stations that outlive the TUI, sockets, Docker.
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
cd ocppd-tui && gofmt -l . && go vet ./... && go test ./...
```

The TUI is a Go module in `ocppd-tui/`, built on
[Bubble Tea](https://github.com/charmbracelet/bubbletea). Rust unit tests sit
next to the code they test, as `<file>.test.rs`, and Go tests as `<file>_test.go`. Test fixtures
(a fake CSMS, a test certificate authority) live in `ocppd-core/src/fixtures`.
The only integration tests, in `ocppd/tests`, run the binary.

## License

Apache-2.0
