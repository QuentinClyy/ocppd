package command

import "encoding/json"

type Target struct {
	Station   string
	Connector int
}

var connectorCommands = map[string]string{
	"p": "plug_in",
	"s": "stop",
	"u": "unplug",
	"f": "fault",
	"c": "clear_fault",
}

func Driver(key string, at Target) (string, bool) {
	if key == "x" {
		return Encode(map[string]any{"command": "power_cut", "station": at.Station}), true
	}
	name, ok := connectorCommands[key]
	if !ok || at.Connector == 0 {
		return "", false
	}
	return Encode(map[string]any{"command": name, "station": at.Station, "connector": at.Connector}), true
}

func Authorize(at Target, idTag string) string {
	return Encode(map[string]any{"command": "authorize", "station": at.Station, "connector": at.Connector, "id_tag": idTag})
}

func Remove(station string) string {
	return Encode(map[string]any{"command": "remove", "station": station})
}

func Describe(station string) string {
	return Encode(map[string]any{"command": "describe", "station": station})
}

func Configure(station, key, value string) string {
	return Encode(map[string]any{"command": "configure", "station": station, "key": key, "value": value})
}

func Encode(fields map[string]any) string {
	line, _ := json.Marshal(fields)
	return string(line)
}
