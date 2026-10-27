package popup

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
)

func keyMsg(key string) tea.KeyPressMsg {
	codes := map[string]rune{"enter": tea.KeyEnter, "esc": tea.KeyEscape, "tab": tea.KeyTab, "backspace": tea.KeyBackspace}
	if code, ok := codes[key]; ok {
		return tea.KeyPressMsg{Code: code}
	}
	return tea.KeyPressMsg{Code: []rune(key)[0], Text: key}
}

func spawnKeys(f *Spawn, keys ...string) (string, bool) {
	var line string
	var closed bool
	for _, key := range keys {
		line, closed = f.Key(keyMsg(key))
	}
	return line, closed
}

func twoConnectors() fleet.Station {
	var f fleet.Fleet
	f.Apply(`{"station":"CP1","at":1,"kind":"status_changed","connector":1,"status":"Available"}`)
	f.Apply(`{"station":"CP1","at":1,"kind":"status_changed","connector":2,"status":"Available"}`)
	return f.Stations[0]
}

func stationKeys(p Station, keys ...string) Result {
	r := Result{Popup: &p}
	for _, key := range keys {
		r = r.Popup.Key(keyMsg(key), twoConnectors())
	}
	return r
}
