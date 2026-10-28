# Simulation

ocppd speaks the real protocol over a real WebSocket. Everything behind the
station is simulated.

## The station

- **Power:** by default a station has one connector, rated 230 V × 32 A × 3
  phases (22 kW); the [station model](configuration.md#station-model) changes
  that. A 2.0.1 station has one EVSE per connector, of type `cType2`.
- **EV charging:** an EV charges at its own maximum, within the station's
  limits and the CSMS's charging profiles. The curve depends on the station's
  `current`:
  - `ac`: the car's onboard charger sets the pace. Power is flat up to 97 %
    and tapers after that.
  - `dc`: the car's battery management sets the pace. Power is flat up to
    50 %, falls to half at 80 % and to 5 % at 100 %.
  - A station limit scales the curve down; it does not change its shape.
  - A `dc` station is three-phase on the grid side. Meter values stay on that
    side: there is no DC voltage or current, and no converter loss.
- **Meter values:** the meter values and the EVSE `Power` variable follow that
  curve.
- **Time:** the clock is the host's.

## What is simulated

- **Hardware events** (plug in, card, fault, lock failure, power cut) are
  [commands](usage.md#commands).
- **Transfers** never leave the machine. A firmware update, diagnostics
  upload or log upload only needs a valid url scheme and takes 10 seconds.
  `fail_transfer` makes the next one fail.
  - A failing log upload is retried as the CSMS asks.
  - Firmware retries and time windows are ignored.
  - The new firmware version is the last segment of the url.
  - A firmware signature only has to be present.
- **Certificates** are real X.509 with EC P-256 keys, kept in memory.
- **State** (settings, certificates, profiles, reservations, monitors, display
  messages, the authorization cache and the local list) lives in memory.
  - A reset or a power cut keeps it.
  - Restarting ocppd does not.
- **The driver's screen** is the event stream: display messages, costs and
  tariffs arrive as events, and no screen is drawn.

## Scale

One process against a CSMS that accepts everything, on 20 cores and 16 GB:

| Stations | Memory | Boot of all | Steady load |
|---|---|---|---|
| 1 000 | 28 MB | under 5 s | |
| 10 000 | 221 MB | under 5 s | heartbeats every 10 s: about 0.01 core |
| 10 000 charging | 250 MB | | meter values every 10 s, 1 200 calls/s: 0.11 core |
| 10 000 OCPP 2.0.1 | 472 MB | 4 s | heartbeats every 10 s: 0.05 core |
| 5 000 1.6 + 5 000 2.0.1 | 370 MB | 4.5 s | heartbeats every 10 s: 0.04 core |

A 2.0.1 station takes about twice the memory of a 1.6 one, for its device
model.

The ceiling is the ephemeral port range. One CSMS address takes at most about
28 000 connections; further stations log `Cannot assign requested address` and
retry. To go further, widen `net.ipv4.ip_local_port_range`, or spread stations
over several processes and CSMS addresses.

To measure your own setup, the repository has a CSMS that accepts everything:

```
cargo run --release --example sink_csms 127.0.0.1:9000 60
```
