package ui

import (
	"slices"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

var paneBindings = map[pane][]draw.Binding{
	fleetPane: {
		{Group: "stations", Keys: "j / k", Help: "next / previous row", Bar: "move"},
		{Group: "stations", Keys: "enter", Help: "open / close station", Bar: "open"},
		{Group: "stations", Keys: "space", Help: "station details", Bar: "details"},
		{Group: "stations", Keys: "gg / G", Help: "first / last row"},
		{Group: "stations", Keys: "d", Help: "remove the station (dd: no question)"},
		{Group: "driver", Keys: "p", Help: "plug in an EV", Bar: "plug"},
		{Group: "driver", Keys: "a", Help: "present an id tag", Bar: "auth"},
		{Group: "driver", Keys: "s", Help: "stop the transaction", Bar: "stop tx"},
		{Group: "driver", Keys: "u", Help: "unplug", Bar: "unplug"},
		{Group: "driver", Keys: "f", Help: "fault the connector"},
		{Group: "driver", Keys: "c", Help: "clear the fault"},
		{Group: "driver", Keys: "x", Help: "power cut, then reboot"},
		{Group: "driver", Keys: "o", Help: "take offline / bring online", Bar: "link"},
	},
	logPane: {
		{Group: "log", Keys: "j / k", Help: "next / previous line", Bar: "move"},
		{Group: "log", Keys: "gg / G", Help: "oldest / newest line"},
		{Group: "log", Keys: "V / v", Help: "select lines, then y copies them"},
		{Group: "log", Keys: "yy / Y", Help: "copy the line"},
	},
}

var globalBindings = []draw.Binding{
	{Group: "panes", Keys: "^hjkl", Help: "focus the pane that way (or ^arrows)", Bar: "pane"},
	{Group: "panes", Keys: "+ / -", Help: "grow / shrink the pane (or drag)"},
	{Group: "panes", Keys: "=", Help: "reset the split"},
	{Group: "panes", Keys: "z", Help: "zoom the pane / show both", Bar: "zoom"},
	{Group: "panes", Keys: "/", Help: "filter the pane (stations: id or ocpp)", Bar: "filter"},
	{Group: "general", Keys: "drag", Help: "select and copy (shift: terminal selects)"},
	{Group: "general", Keys: "5j", Help: "a count repeats a move; 12G goes to row 12"},
	{Group: "general", Keys: "^d / ^u", Help: "scroll log down / up (or wheel)"},
	{Group: "general", Keys: "tab", Help: "log: selected station / all", Bar: "log"},
	{Group: "general", Keys: "n", Help: "spawn stations", Bar: "spawn"},
	{Group: "general", Keys: "q", Help: "quit; the stations keep running", Bar: "quit"},
	{Group: "general", Keys: "Q", Help: "stop the daemon and every station"},
	{Group: "general", Keys: "esc / ?", Help: "close this help"},
}

func bindingsFor(p pane) []draw.Binding {
	return append(slices.Clone(paneBindings[p]), globalBindings...)
}
