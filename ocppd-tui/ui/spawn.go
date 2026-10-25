package ui

import (
	"errors"
	"strconv"
	"strings"
)

func (f *form) line() (string, error) {
	fields := map[string]any{"command": "spawn", "ocpp": f.ocpp.options[f.ocpp.index]}
	if f.isListed() {
		var stations []map[string]string
		for _, s := range f.stations {
			id := strings.TrimSpace(s.id)
			if id == "" {
				continue
			}
			entry := map[string]string{"id": id}
			if s.password != "" {
				entry["password"] = s.password
			}
			stations = append(stations, entry)
		}
		if len(stations) == 0 {
			return "", errors.New("add at least one station id")
		}
		fields["stations"] = stations
	} else {
		count, err := whole(or(f.count, "1"))
		if err != nil || count < 1 {
			return "", errors.New("count must be a whole number from 1")
		}
		fields["prefix"], fields["count"] = or(f.prefix, "CP"), count
		if f.password != "" {
			fields["password"] = f.password
		}
	}
	for key, value := range map[string]string{"vendor": f.vendor, "model": f.model, "csms": f.csms} {
		if text := strings.TrimSpace(value); text != "" {
			fields[key] = text
		}
	}
	numbers := []struct {
		key, label, text string
		isWhole          bool
	}{
		{"phases", "phases", f.phases, true},
		{"max_current_a", "max A", f.maxCurrent, false},
		{"connectors", "connectors", f.connectors, true},
	}
	for _, n := range numbers {
		if strings.TrimSpace(n.text) == "" {
			continue
		}
		value, err := parseNumber(n.text, n.isWhole)
		if err != nil {
			return "", errors.New(n.label + " must be a number")
		}
		fields[n.key] = value
	}
	return encode(fields), nil
}

func parseNumber(text string, isWhole bool) (any, error) {
	if isWhole {
		return whole(text)
	}
	return strconv.ParseFloat(strings.TrimSpace(text), 64)
}

func whole(text string) (int, error) {
	return strconv.Atoi(strings.TrimSpace(text))
}

func or(text, fallback string) string {
	if strings.TrimSpace(text) == "" {
		return fallback
	}
	return strings.TrimSpace(text)
}
