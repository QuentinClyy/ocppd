package ui

import (
	"encoding/json"
	"slices"
	"strings"
)

type setting struct {
	Key      string
	Readonly bool
	Value    *string
}

type description struct {
	Kind            string
	Station         string
	Vendor          string
	Model           string
	FirmwareVersion string `json:"firmware_version"`
	Csms            string
	Configuration   []setting
}

func parseDescription(line string) (description, bool) {
	var parsed description
	if json.Unmarshal([]byte(line), &parsed) != nil || parsed.Kind != "station_described" {
		return description{}, false
	}
	return parsed, true
}

type change struct {
	Station   string
	Kind      string
	Key       string
	Component string
	Variable  string
	Value     string
	Reason    string
	Connector *int
}

func (d description) with(key, value string) (description, bool) {
	index := slices.IndexFunc(d.Configuration, func(s setting) bool { return strings.EqualFold(s.Key, key) })
	if index < 0 {
		return d, false
	}
	d.Configuration = slices.Clone(d.Configuration)
	d.Configuration[index].Value = &value
	return d, true
}

func (m Model) track(line string) Model {
	var c change
	if m.details == nil || json.Unmarshal([]byte(line), &c) != nil || c.Station != m.details.station {
		return m
	}
	next := *m.details
	switch c.Kind {
	case "intent_rejected":
		if c.Connector != nil {
			return m
		}
		next.config.err = c.Reason
	case "configuration_changed", "variable_changed":
		key := c.Key
		if c.Kind == "variable_changed" {
			key = c.Component + "." + c.Variable
		}
		described, found := next.described.with(key, c.Value)
		if !found {
			return m.sendLine(describeLine(c.Station))
		}
		next.described, next.config.err = described, ""
	default:
		return m
	}
	m.details = &next
	return m
}

func describeLine(station string) string {
	return encode(map[string]any{"command": "describe", "station": station})
}
