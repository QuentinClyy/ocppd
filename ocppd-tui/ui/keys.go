package ui

import "encoding/json"

type binding struct {
	group string
	keys  string
	help  string
	bar   string
}

var bindings = []binding{
	{"move", "j / k", "next / previous row", ""},
	{"move", "enter", "open / close station", "open"},
	{"move", "gg / G", "first / last row", ""},
	{"move", "^d / ^u", "scroll log down / up (or wheel)", ""},
	{"pane", "^hjkl", "focus the pane that way (or ^arrows)", ""},
	{"pane", "+ / -", "grow / shrink the pane (or drag)", ""},
	{"pane", "=", "reset the split", ""},
	{"pane", "z", "zoom the pane / show both", ""},
	{"driver", "p", "plug in an EV", "plug"},
	{"driver", "a", "present an id tag", "auth"},
	{"driver", "s", "stop the transaction", "stop tx"},
	{"driver", "u", "unplug", "unplug"},
	{"driver", "f", "fault the connector", "fault"},
	{"driver", "c", "clear the fault", "clear"},
	{"driver", "x", "power cut, then reboot", "cut"},
	{"fleet", "n", "spawn stations", "spawn"},
	{"fleet", "d", "remove the station (dd: no question)", "remove"},
	{"fleet", "tab", "log: this station / all", "log"},
	{"fleet", "q", "quit; the stations keep running", "quit"},
	{"fleet", "Q", "stop the daemon and every station", ""},
	{"fleet", "esc / ?", "close this help", ""},
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
