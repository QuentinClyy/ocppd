package ui

import "slices"

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
		{"stations", "space", "station details", "details"},
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
		{"log", "j / k", "next / previous line", "move"},
		{"log", "gg / G", "oldest / newest line", ""},
		{"log", "V / v", "select lines, then y copies them", ""},
		{"log", "yy / Y", "copy the line", ""},
	},
}

var globalBindings = []binding{
	{"panes", "^hjkl", "focus the pane that way (or ^arrows)", "pane"},
	{"panes", "+ / -", "grow / shrink the pane (or drag)", ""},
	{"panes", "=", "reset the split", ""},
	{"panes", "z", "zoom the pane / show both", "zoom"},
	{"panes", "/", "filter the pane (stations: id or ocpp)", "filter"},
	{"general", "5j", "a count repeats a move; 12G goes to row 12", ""},
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
