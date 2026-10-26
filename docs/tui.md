# Terminal UI

`ocppd-tui` is an interactive view of the fleet. Run ocppd in a terminal and it
opens the TUI, as long as `ocppd-tui` sits beside ocppd or on the `PATH`. ocppd
starts a daemon in the background first, if none is running, and the TUI
connects to it:

```
ocppd --csms ws://localhost:8180/steve/websocket/CentralSystemService
```

The TUI talks to the daemon with the same JSON lines as a script (see
[Usage](usage.md)), so anything the TUI does, a script can do too. Quitting
leaves the daemon and its stations running: run `ocppd` again to come back to
them, or press `Q` to stop everything. `ocppd-tui --connect tcp:<host>:<port>`
reaches a daemon elsewhere, with `OCPPD_TOKEN` set.

## The screen

```
 ocppd  ws://localhost:8180/steve/websocket/CentralSystemService    2 stations
 ───────────────────────────────────────────────────────────────────────────────
   STATION               OCPP   LINK  BOOT      STATUS       kW     SOC   TX
 ▾ EVB-0001              1.6    up    Accepted  1 charging   11.0
   ├─ 1                                         Charging     11.0   47%   1042
   └─ 2                                         Available    —      —
 ▸ CS1                   2.0.1  up    Accepted               —
 ───────────────────────────────────────────────────────────────────────────────
 all stations · log
 14:02:12  EVB-0001   transaction_started    connector=1 transaction_id=1042
 14:02:42  EVB-0001   meter_sample           connector=1 power_w=11000 soc=0.47
 ───────────────────────────────────────────────────────────────────────────────
 j/k move  enter open  p plug  a auth  s stop tx  u unplug  f fault  ? help
```

- **Stations** are listed by id, collapsed. Open one to see its connectors (a
  2.0.1 EVSE shows as one connector). A collapsed station shows its most
  important status and its total power.
- **The log** shows every event, or only the selected station's with `tab`.
- From 150 columns, the log sits beside the stations instead of under them.
- A `▐` scrollbar shows on a pane that has more lines than fit.

## Keys

Driver keys act on the selected connector. On a station row, they act on its
only connector, or open the station when it has several.

| Key | Action |
|---|---|
| `j` / `k` | next / previous row |
| `enter` | open / close a station; on a connector, close its station |
| `l` / `h` | open / close a station, vim style |
| `gg` / `G` | first / last row |
| `^d` / `^u` | scroll the log down / up; the mouse wheel scrolls the pane under it |
| `p` | plug in an EV |
| `a` | present an id tag: type it, `enter` sends, `esc` cancels |
| `s` | stop the transaction |
| `u` | unplug |
| `f` | fault the connector |
| `c` | clear the fault |
| `x` | power cut, then reboot |
| `n` | spawn stations (the form below) |
| `d` | remove the station: `y` confirms, `dd` skips the question |
| `tab` | log: the selected station / all stations |
| `q` | quit; the daemon and its stations keep running |
| `Q` | stop the daemon and every station: `y` confirms |
| `esc` / `?` | open / close the key help |

The TUI captures the mouse so the wheel scrolls its panes; hold Shift to select
text.

## Spawn form

`n` opens a form that builds a spawn line (see
[Configuration](configuration.md)):

- **stations**: a list of ids, each with its own password (`ctrl+n` adds one),
  or a prefix and a count sharing one password. `←` / `→` switches.
- **ocpp**, **vendor**, **model**, **phases**, **max A**, **connectors**,
  **csms**: an empty field keeps ocppd's default, shown in grey.

`tab` / `⇧tab` move between fields, `enter` spawns, `esc` cancels. Passwords
show as `•`.
