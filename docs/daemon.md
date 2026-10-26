# Daemon

A daemon is ocppd running with `--listen`. It keeps its stations running while
clients come and go. The [TUI](tui.md) is one client; a script, a web bridge or
another machine can be others. Every client speaks the same JSON lines as
stdin and stdout (see [Usage](usage.md)).

## Starting it

Run `ocppd` in a terminal and it starts a daemon for you, if none is running,
then opens the TUI on it. `q` leaves the daemon running, and running `ocppd`
again comes back to the same stations.

To start one yourself, under tmux, systemd or Docker:

```
ocppd --csms ws://localhost:8180/steve/websocket/CentralSystemService \
  --listen unix:$XDG_RUNTIME_DIR/ocppd.sock
```

`--listen` can be given several times. A daemon never reads stdin.

| Address | Who can connect |
|---|---|
| `unix:<path>` | the same user: the socket is created with mode 0600 |
| `tcp:<host>:<port>` | anyone who can reach the port and knows `OCPPD_TOKEN` |

A tcp listener refuses to start without `OCPPD_TOKEN`. A tcp client must send
it first, within 5 seconds:

```
{"command":"hello","token":"s3cret"}
```

## Files

| What | Where |
|---|---|
| socket that `ocppd` and the TUI use | `OCPPD_SOCKET`, else `$XDG_RUNTIME_DIR/ocppd.sock`, else `~/.ocppd.sock` |
| log of a daemon started by `ocppd` | `$XDG_STATE_HOME/ocppd/ocppd.log`, else `~/.local/state/ocppd/ocppd.log` |

## What a client receives

1. One line about the daemon:
   ```
   {"kind":"daemon","csms":"ws://…","version":"0.2.0"}
   ```
2. A snapshot: the latest event of each kind for each station and connector,
   in time order. It is enough to rebuild the state of the fleet.
3. Every event from then on.

A command that cannot run is answered with `command_rejected`, to the client
that sent it only. A client that falls more than 10,000 lines behind is
disconnected.

## Stopping it

`{"command":"shutdown"}`, `Q` in the TUI, SIGTERM or Ctrl+C stop the daemon
and every station, and remove its socket. Each station first stops its
transactions (reason `Other`), waits for the CSMS to answer, and closes its
WebSocket. A station that cannot finish within 5 seconds, offline for example,
is cut off.

## From a script

Any tool that speaks lines over a socket is a client:

```
socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/ocppd.sock
```

## Docker

The image runs a daemon on tcp port 7600:

```
docker run -d -p 127.0.0.1:7600:7600 \
  -e OCPPD_TOKEN=s3cret -e OCPPD_CSMS=ws://csms.example/ocpp \
  ghcr.io/quentinclyy/ocppd
OCPPD_TOKEN=s3cret ocppd-tui --connect tcp:127.0.0.1:7600
```

To build it from a checkout, put the static binary for each architecture in
`dist/<arch>/ocppd` (`amd64`, `arm64`), then:

```
docker build -t ocppd .
```
