# Usage

```
ocppd --csms <url>
```

| Option | Meaning |
|---|---|
| `--csms <url>` | The CSMS WebSocket url. A station connects to `<url>/<station id>`. Falls back to the `OCPPD_CSMS` environment variable. |
| `-h`, `--help` | Print the help and exit. |
| `-V`, `--version` | Print the version and exit. |

ocppd reads commands from stdin, one JSON object per line, and writes events to
stdout, one JSON object per line. Logs and errors go to stderr. It exits when
stdin is closed and every station has stopped.

| Exit status | Meaning |
|---|---|
| 0 | stdin closed and every station stopped |
| 1 | runtime error |
| 2 | usage error |

Because both directions are JSON lines, ocppd fits in a pipeline:

```
mkfifo commands
ocppd --csms ws://localhost:8081 < commands | jq 'select(.kind == "transaction_started")' &
exec 3> commands
echo '{"command":"spawn","prefix":"CS","count":10,"ocpp":"2.0.1"}' >&3
```

## Commands

| Command | Effect |
|---|---|
| `{"command":"spawn","prefix":"CP","count":2}` | start stations `CP1` and `CP2` (see [Configuration](configuration.md) for every option) |
| `{"command":"plug_in","station":"CP1","connector":1}` | plug an EV into connector 1 |
| `{"command":"authorize","station":"CP1","id_tag":"TAG1"}` | present a card; a transaction starts once the CSMS accepts it and an EV is plugged in (`"connector":1` picks the connector) |
| `{"command":"stop","station":"CP1","connector":1}` | stop the transaction from the station |
| `{"command":"unplug","station":"CP1","connector":1}` | unplug the EV; a running transaction stops |
| `{"command":"fault","station":"CP1","connector":1}` | the connector goes `Faulted`: its transaction stops and it refuses new ones |
| `{"command":"lock_failure","station":"CP1","connector":1}` | the cable lock breaks: unlocking fails and plugging in faults the connector |
| `{"command":"clear_fault","station":"CP1","connector":1}` | clear the fault and repair the lock |
| `{"command":"power_cut","station":"CP1"}` | the power fails: transactions stop, the station goes down and boots again with its state |
| `{"command":"fail_transfer","station":"CP1","stage":"download"}` | make the next firmware download fail; other stages are `signature`, `install` and `upload` (diagnostics or log) |

`plug_in` takes an optional EV. These are the defaults:

```
"ev":{"capacity_kwh":60,"soc":0.2,"max_kw":11,"target_soc":1.0}
```

A command that does not fit (an unknown station, an EV already plugged in, a
stop without a transaction) is refused with an `intent_rejected` event.

## Events

Every event has the station id, the time in Unix milliseconds and a `kind`:

```
{"station":"CP1","at":1792893141250,"kind":"boot_status","status":"Accepted"}
```

The main kinds:

| Kind | When |
|---|---|
| `connected`, `disconnected` | the WebSocket opens or closes |
| `boot_status` | the CSMS answers the boot |
| `status_changed` | a connector changes status |
| `authorization` | the CSMS answers a card |
| `transaction_started`, `transaction_stopped` | a transaction begins or ends |
| `meter_sample` | a transaction's meter is sampled: energy, power and SoC (`connector` in 1.6, `evse` in 2.0.1) |
| `configuration_changed`, `variable_changed` | the CSMS changes a setting (secrets are never echoed) |
| `charging_limit_changed` | a charging profile changes the power |
| `firmware_status`, `log_status` | a firmware update or an upload progresses |
| `monitor_triggered` | a 2.0.1 variable monitor fires |
| `display_message_set`, `cost_shown`, `driver_message` | what a 2.0.1 driver would see on screen |
| `call_retried`, `call_abandoned`, `intent_rejected` | a message is retried or given up, or a command is refused |
