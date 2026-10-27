package ui

import "encoding/json"

type greeting struct {
	Kind    string
	Csms    string
	Version string
}

func parseGreeting(line string) (greeting, bool) {
	var parsed greeting
	if json.Unmarshal([]byte(line), &parsed) != nil || parsed.Kind != "daemon" {
		return greeting{}, false
	}
	return parsed, true
}
