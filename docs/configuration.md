# Configuration

Stations are configured when they are spawned.

```
{"command":"spawn","prefix":"CP","count":100}
```

| Field | Default | Meaning |
|---|---|---|
| `prefix`, `count` | required | station ids are `<prefix>1` to `<prefix><count>` |
| `ocpp` | `"1.6"` | `"1.6"` or `"2.0.1"` |
| `csms` | `--csms` | another CSMS url for this batch, so one process can run a mixed fleet |
| `jitter` | `true` | spread heartbeats and reconnects; `false` aligns the whole fleet, to test load storms |
| `configuration` | none | settings applied before the first connection; a refused one stops the spawn |
| `password` | none | a plain-text basic auth password |

## Settings

A 1.6 station takes its configuration keys, with the same rules as
ChangeConfiguration:

```
"configuration":{"HeartbeatInterval":"60","MeterValueSampleInterval":"30"}
```

A 2.0.1 station takes device model variables as `Component.Variable`, with
optional instances as `Component[instance].Variable[instance]`:

```
"configuration":{"OCPPCommCtrlr.OfflineThreshold":"120"}
```

## Security profiles

| Profile | Transport | Authentication |
|---|---|---|
| 0 | `ws://` | none |
| 1 | `ws://` | basic auth |
| 2 | `wss://`, TLS 1.2 or newer | basic auth |
| 3 | `wss://`, TLS 1.2 or newer | client certificate |

1.6:

```
{"command":"spawn","prefix":"CP","count":1,
 "configuration":{"AuthorizationKey":"30313233343536373839616263646566","SecurityProfile":"2"}}
```

`AuthorizationKey` is the password in hex. `"password"` takes it in plain text
instead.

2.0.1:

```
{"command":"spawn","prefix":"CS","count":1,"ocpp":"2.0.1",
 "configuration":{"SecurityCtrlr.SecurityProfile":"1","SecurityCtrlr.BasicAuthPassword":"correct-horse-battery"}}
```

The CSMS certificate is trusted through the system store, or through
`SSL_CERT_FILE` (a PEM file that then replaces the system store), plus any
root the CSMS installs.

For profile 3, the station needs a certificate signed by the CSMS. It gets one
when the CSMS triggers a certificate signing request:
- ExtendedTriggerMessage `SignChargePointCertificate` for 1.6;
- TriggerMessage `SignChargingStationCertificate` for 2.0.1.

Without one the station does not connect.

## Reconnection

A station that loses the CSMS reconnects with a growing, jittered delay, and
keeps its transaction messages to send once it is back. A 2.0.1 station
follows the `RetryBackOffWaitMinimum`, `RetryBackOffRandomRange` and
`RetryBackOffRepeatTimes` variables, and its network profiles: the CSMS can add
a profile and change their priority, and the station falls back to the next
one when a profile keeps failing.
