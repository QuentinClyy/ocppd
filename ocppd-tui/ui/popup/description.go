package popup

import (
	"encoding/json"
	"slices"
	"strings"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
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

func IsDescription(line string) bool {
	_, ok := parseDescription(line)
	return ok
}

func (p Station) Observe(line string) (Station, []string) {
	if d, ok := parseDescription(line); ok {
		if d.Station == p.id {
			p.described = d
		}
		return p, nil
	}
	var c change
	if json.Unmarshal([]byte(line), &c) != nil || c.Station != p.id {
		return p, nil
	}
	switch c.Kind {
	case "intent_rejected":
		if c.Connector == nil {
			p.config.err = c.Reason
		}
	case "csms_changed":
		p.config.err = ""
		return p, []string{command.Describe(c.Station)}
	case "configuration_changed", "variable_changed":
		key := c.Key
		if c.Kind == "variable_changed" {
			key = c.Component + "." + c.Variable
		}
		described, found := p.described.with(key, c.Value)
		if !found {
			return p, []string{command.Describe(c.Station)}
		}
		p.described, p.config.err = described, ""
	}
	return p, nil
}
