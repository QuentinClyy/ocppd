# Simulation

ocppd speaks the real protocol over a real WebSocket. Everything behind the
station is simulated.

## The station

- **Power:** by default a station has one connector, rated 230 V × 32 A × 3
  phases (22 kW); the [station model](configuration.md#station-model) changes
  that. A 2.0.1 station has one EVSE per connector, of type `cType2`, or
  `cCCS2` for `dc`.
- **The battery:** its voltage rises with the state of charge, from 0.90 to
  1.15 × the EV's `pack_v`. It accepts its peak current up to 20 %, a little
  less as it fills, and from 60 % a current that falls in a straight line to
  C/20 at 100 % (constant voltage).
- **The current** is the lowest of what the battery accepts and what the
  station supplies:
  - `ac`: the onboard charger draws its `ac_kw` over `ac_phases` phases, within
    the station's limit on those phases. The battery only limits it near 100 %,
    so power is flat almost to the end.
  - `dc`: the station's grid power, minus 5 % conversion loss, at the battery
    voltage, up to `max_output_a`. On a current limit, power rises with the
    battery voltage; on a power limit, it is flat; then the battery tapers it.
    An 800 V EV draws twice the power of a 400 V one at the same current.
  - CSMS charging profiles lower the station's limit.
- **DC start-up:** 5 s of cable check and pre-charge without current, then the
  current ramps at 50 A/s. A limit change during the session does not ramp.
- **Meter values:** the meter values and the EVSE `Power` variable follow that
  current.
  - `ac`: voltage on every phase of the station, current only on the phases
    the EV uses.
  - `dc`: at the `Outlet`, the battery voltage and the DC current, without a
    phase; at the `Inlet`, the grid voltage and current on each phase, and the
    grid power with the loss.
  - `SoC` only on a `dc` station, where the EV sends it during the DC
    session (DIN SPEC 70121, ISO 15118-2). An `ac` station has no data link
    to the EV: it refuses `SoC` as a measurand.
  - Voltage and current carry a small noise (±0.5 %, ±1 %), the same at the
    same time for the same station; a station spawned with `jitter` `false`
    has none. The energy register is exact.
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
