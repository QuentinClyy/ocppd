package ui

import (
	"encoding/json"
	"slices"
)

type binding struct {
	group string
	keys  string
	help  string
	bar   string
}

var paneBindings = map[pane][]binding{
	fleetPane: {
		{"stations", "j / k", "next / previous row", "move"},
		{"stations", "enter", "open / close station", "open"},
		{"stations", "gg / G", "first / last row", ""},
		{"stations", "d", "remove the station (dd: no question)", ""},
		{"driver", "p", "plug in an EV", "plug"},
		{"driver", "a", "present an id tag", "auth"},
		{"driver", "s", "stop the transaction", "stop tx"},
		{"driver", "u", "unplug", "unplug"},
		{"driver", "f", "fault the connector", ""},
		{"driver", "c", "clear the fault", ""},
		{"driver", "x", "power cut, then reboot", ""},
	},
	logPane: {
		{"log", "j / k", "scroll down / up", "scroll"},
		{"log", "gg / G", "oldest / newest line", ""},
	},
}

var globalBindings = []binding{
	{"panes", "^hjkl", "focus the pane that way (or ^arrows)", "pane"},
	{"panes", "+ / -", "grow / shrink the pane (or drag)", ""},
	{"panes", "=", "reset the split", ""},
	{"panes", "z", "zoom the pane / show both", "zoom"},
	{"general", "^d / ^u", "scroll log down / up (or wheel)", ""},
	{"general", "tab", "log: selected station / all", "log"},
	{"general", "n", "spawn stations", "spawn"},
	{"general", "q", "quit; the stations keep running", "quit"},
	{"general", "Q", "stop the daemon and every station", ""},
	{"general", "esc / ?", "close this help", ""},
}

func bindingsFor(p pane) []binding {
	return append(slices.Clone(paneBindings[p]), globalBindings...)
}

var connectorCommands = map[string]string{
	"p": "plug_in",
	"s": "stop",
	"u": "unplug",
	"f": "fault",
	"c": "clear_fault",
}

type target struct {
	station   string
	connector int
}

func command(key string, at target) (string, bool) {
	if key == "x" {
		return encode(map[string]any{"command": "power_cut", "station": at.station}), true
	}
	name, ok := connectorCommands[key]
	if !ok || at.connector == 0 {
		return "", false
	}
	return encode(map[string]any{"command": name, "station": at.station, "connector": at.connector}), true
}

func authorize(at target, idTag string) string {
	return encode(map[string]any{"command": "authorize", "station": at.station, "connector": at.connector, "id_tag": idTag})
}

func encode(fields map[string]any) string {
	line, _ := json.Marshal(fields)
	return string(line)
}
