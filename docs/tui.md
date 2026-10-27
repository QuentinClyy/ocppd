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
 j/k move  enter open  p plug  a auth  s stop tx  u unplug  ^hjkl pane  ? help
```

- **Stations** are listed by id, collapsed. Open one to see its connectors (a
  2.0.1 EVSE shows as one connector). A collapsed station shows its most
  important status and its total power.
- **The log** shows every event, or only the selected station's with `tab`.
- From 150 columns, the log sits beside the stations instead of under them.
- A `▐` scrollbar shows on a pane that has more lines than fit.
- One pane has the focus: its title is bold, and the other's is grey. The key
  bar and the help (`?`) show the focused pane's keys and the global ones.

## Keys

Driver keys act on the selected connector, from the stations pane. On a
station row, they act on its only connector, or open the station when it has
several. In the log pane, `j` / `k` scroll a line, `gg` goes to the oldest
line and `G` back to the newest.

| Key | Action |
|---|---|
| `j` / `k` | next / previous row |
| `enter` | open / close a station; on a connector, close its station |
| `space` | station details (below) |
| `gg` / `G` | first / last row |
| `^d` / `^u` | scroll the log down / up; the mouse wheel scrolls the pane under it |
| `^hjkl` | focus the pane left / below / above / right; `^←` `^↓` `^↑` `^→` too |
| `+` / `-` | grow / shrink the focused pane |
| `=` | reset the split |
| `z` | zoom the focused pane, hiding the other; again to show both |
| `/` | filter the focused pane: stations by id or OCPP version (`2.0.1`), the log by text; `enter` keeps the filter, `esc` clears it |
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
| `esc` / `?` | open / close the key help; `j` / `k` scroll it on a short terminal |

The TUI captures the mouse: the wheel scrolls the pane under it, a click
focuses a pane and selects a row, a click on the selected row opens or closes
it, and dragging the divider resizes the panes. Hold Shift to select text.

## Station details

`space` opens a station's details: its OCPP version, link, boot status and
when it connected, each connector's status, power, state of charge, meter
reading and transaction, its vendor, model, firmware and CSMS url, and its
last three events. `j` / `k` pick a connector and the driver keys act on it;
`esc` or `q` close the details.

`tab` switches to the station's configuration: its 1.6 keys or 2.0.1
variables, read-only ones marked `ro`. `j` / `k` pick a key, `/` filters
them, `space` flips a `true` / `false` value, and `i` or `enter` edits one:
`enter` applies it, `esc` cancels. The station takes the change as if the
CSMS had sent it; a refused change shows its reason in the bottom border.

## Spawn form

`n` opens a form that builds a spawn line (see
[Configuration](configuration.md)):

- **entry**: a list of ids, each with its own password (`ctrl+n` adds one),
  or a prefix and a count sharing one password. `←` / `→` switches.
- **ocpp**, **vendor**, **model**, **phases**, **max A**, **connectors**,
  **csms**: an empty field keeps ocppd's default, shown in grey.

`tab` / `⇧tab` move between fields and `enter` spawns. `esc` switches to
normal mode, vim style: `j` / `k` move between fields, `h` / `l` switch a
choice, `gg` / `G` go to the first / last field, `i` or `a` types again,
`enter` spawns, and `esc` or `q` cancels. The box's bottom border shows the
mode and, when the spawn is refused, why. Passwords show as typed. On a small
terminal the form shrinks to fit and scrolls to the focused field.
