# Configuration

Stations are configured when they are spawned. One spawn line describes one
station model, and either numbers its stations or lists them:

```
{"command":"spawn","prefix":"CP","count":100}
{"command":"spawn","vendor":"ACME","model":"WB22","phases":1,"max_current_a":16,
 "configuration":{"SecurityProfile":"1"},
 "stations":[{"id":"EVB-0001","password":"s3cret-0001"},{"id":"EVB-0002","password":"s3cret-0002"}]}
```

## Identities

Use one of the two forms:

| Field | Meaning |
|---|---|
| `prefix`, `count` | station ids are `<prefix>1` to `<prefix><count>` |
| `password` | with `prefix`: one plain-text basic auth password for every station |
| `stations` | a list of `{"id":...,"password":...}`; `password` is optional |

The serial number is the station id. A password is sent from security
profile 1 up, and is never echoed. A station given a password and no
security profile gets profile 1 on `ws://` and 2 on `wss://`.

## Station model

| Field | Default | Meaning |
|---|---|---|
| `ocpp` | `"1.6"` | `"1.6"` or `"2.0.1"` |
| `csms` | `--csms` | another CSMS url for these stations, so one process can run a mixed fleet |
| `vendor`, `model`, `firmware_version` | `"ocppd"`, `"sim"`, the ocppd version | reported at boot |
| `voltage_v`, `max_current_a`, `phases` | `230`, `32`, `1` (`3` for `dc`) | the electrical rating on the grid side, `voltage_v` line to neutral: power is `voltage_v × max_current_a × phases`; `phases` is 1 to 3, and 3 for `dc` |
| `max_output_a` | `500` | a `dc` station's maximum output current |
| `current` | `"ac"` | `"ac"` or `"dc"`; a `dc` station needs `phases` 3, and a 2.0.1 one has `cCCS2` connectors |
| `connectors` | `1` | the number of connectors; a 2.0.1 station has one EVSE per connector |
| `jitter` | `true` | spread heartbeats and reconnects; `false` aligns the whole fleet, to test load storms |
| `connect` | `true` | `false` spawns the station offline: it does not connect until a `connect` command |
| `configuration` | none | settings applied before the first connection; a refused one stops the spawn |

Read-only settings, such as `LocalAuthListMaxLength` or
`DeviceDataCtrlr.ItemsPerMessage[GetVariables]`, can be set here, as an
installer would before the station goes online.

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

### Read-only settings

The CSMS cannot change a read-only setting. ocppd can, while the station is
not connected: at spawn, or with a `configure` command once the station is
offline (see `disconnect` in [Usage](usage.md)). Connected, such a command is
refused with `read-only while connected`. The value is still checked: a
number stays a number.

Settings the station computes itself are refused even offline, with
`derived from the station`: `NumberOfConnectors` (1.6), and in 2.0.1
`ClockCtrlr.DateTime`, the `Entries` counts, `DisplayMessages`,
`AvailabilityState`, `SupplyPhases` and `ActiveNetworkProfile`. Set
`connectors` and `phases` at spawn instead.

A 2.0.1 `SecurityCtrlr.SecurityProfile` set offline also applies to the
station's first network profile, so the next connection uses it.

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

A station taken offline with `disconnect` does not reconnect until `connect`;
a reset or a security profile change still reconnects as usual.
