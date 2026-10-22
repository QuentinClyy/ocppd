# ocppd

A sans-IO OCPP charging-station simulator. It runs station state machines
that a CSMS can connect to, driven by JSON-line commands on stdin, with
events as JSON lines on stdout. OCPP 1.6J first, 2.0.1 later.

## Build

Needs OpenSSL headers for `wss` (`apt install libssl-dev pkg-config`).

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
`ocpp1.6` subprotocol. `spawn` also takes `"ocpp": "2.0.1"` for an OCPP
2.0.1 station (`ocpp2.0.1` subprotocol; so far boot, heartbeat, status, the
device model with its base and custom reports and variable monitors, Reset,
local and remote transactions, meter values, availability, the authorization
cache and the local list, security profiles 1 to 3, network profiles, CA
certificates, GetLog, UpdateFirmware, CustomerInformation, charging profiles
from SetChargingProfile or a remote start, GetCompositeSchedule, and
reservations of a given EVSE, any EVSE or a connector type, display
messages, and the running and final cost) and `"csms"` to send that batch to
another CSMS than `--csms`, so one process can run a mixed fleet. For a
2.0.1 station, `"configuration"` keys use the Part 2 shortcut notation
`Component[instance].Variable[instance]`, e.g.
`{"OCPPCommCtrlr.OfflineThreshold":"120"}`:

```
{"command":"spawn","prefix":"CS","count":1,"ocpp":"2.0.1","csms":"ws://localhost:8081"}
```

### Security profiles

`spawn` takes an optional `"configuration"` object, applied with the same
rules as ChangeConfiguration. A refused key stops the spawn.

```
{"command":"spawn","prefix":"CP","count":1,
 "configuration":{"AuthorizationKey":"30313233343536373839616263646566","SecurityProfile":"2"}}
```

`AuthorizationKey` is the hex-encoded basic auth password, sent at
profiles 1 and 2. A 2.0.1 station takes `SecurityCtrlr.SecurityProfile` and
`SecurityCtrlr.BasicAuthPassword` (16 to 40 characters, sent as is) instead. From profile 2 the CSMS url must be `wss://`, over TLS 1.2 or newer. The CSMS
certificate is trusted through the system store, or `SSL_CERT_FILE` (a PEM
file that then replaces the system store), plus any root the CSMS installed
with InstallCertificate. A 2.0.1 station also takes
`SecurityCtrlr.AdditionalRootCertificateCheck`: a new CSMS root must then be
signed by the current one, which stays trusted until the next connection.
A 2.0.1 station keeps its spawn CSMS url and security profile as network profile
slot 0. The CSMS can set slot 1 with SetNetworkProfile and change
`OCPPCommCtrlr.NetworkConfigurationPriority`; after a reboot the station
connects with the first slot, falls back to the next one after
`NetworkProfileConnectionAttempts` failures, and takes the new security
profile once it is connected. Profile 3 presents the charge point certificate
the CSMS signed after an ExtendedTriggerMessage(SignChargePointCertificate),
or a TriggerMessage(SignChargingStationCertificate) for a 2.0.1 station;
without one the station does not connect.

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

## Load

One process, measured against `cargo run --release --example sink_csms
127.0.0.1:9000 60` (a CSMS that accepts everything) on 20 cores, 16 GB:

| Stations | Memory | Boot of all | Steady load |
|---|---|---|---|
| 1 000 | 28 MB | under 5 s | |
| 10 000 | 221 MB | under 5 s | aligned heartbeats every 10 s: about 0.01 core |
| 10 000 charging | 250 MB | | MeterValues every 10 s, 1 200 calls/s: 0.11 core |
| 20 000 | 446 MB | 15 s | |
| 1 000 OCPP 2.0.1 | 55 MB | under 1 s | |
| 10 000 OCPP 2.0.1 | 472 MB | 4 s | heartbeats every 10 s: 0.05 core |

A 2.0.1 station takes about twice the memory of a 1.6 one: each keeps its
own device model of about 80 variables.

The ceiling is the ephemeral port range: one CSMS address takes at most
28 232 connections (`net.ipv4.ip_local_port_range` 32768–60999); further
stations log `Cannot assign requested address` and retry with backoff. As
the range fills, the kernel's port search, not ocppd, costs the CPU. Widen
the range or spread stations over several processes and CSMS addresses to
go further.

## Known limitations

OCPP 1.6J with the Security Whitepaper (edition 4) is complete. The station
simulates, it does not transfer:

- UpdateFirmware, SignedUpdateFirmware, GetDiagnostics and GetLog never
  download or upload anything; `retries`, `retryInterval` and time windows
  are ignored; the new firmware version is the last segment of the URL, and
  a firmware signature only has to be present.
- Certificates, the station key and its certificate live in memory (they
  survive a Reset, not a restart). Keys are EC P-256; only self-signed roots
  are installed.
- Off through configuration keys: reservations on connector 0
  (`ReserveConnectorZeroSupported`), a main meter, offline transactions for
  unknown id tags (`AllowOfflineTxForUnknownId` is not offered).
- `minChargingRate` is ignored and the authorization cache has no size
  limit.

Open protocol gaps:

- UnlockConnector always answers `Unlocked`.
- An unanswered SignCertificate is not resent (whitepaper A02.FR.17–19);
  CertificateSigned does not check the leaf's CN/O.
- SetChargingProfile does not validate the schedule's content.
- No fallback to the previous security profile when the new one cannot
  connect; DeleteCertificate never answers `Failed`.
- An offline transaction whose StartTransaction is abandoned is never
  reported to the CSMS.

