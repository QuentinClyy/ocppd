package popup

import (
	"errors"
	"strconv"
	"strings"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
)

func (f *Spawn) line() (string, error) {
	fields := map[string]any{"command": "spawn", "ocpp": f.ocpp.options[f.ocpp.index]}
	if f.isListed() {
		var stations []map[string]string
		for _, s := range f.stations {
			id := strings.TrimSpace(s.id.String())
			if id == "" {
				continue
			}
			entry := map[string]string{"id": id}
			if password := s.password.String(); password != "" {
				entry["password"] = password
			}
			stations = append(stations, entry)
		}
		if len(stations) == 0 {
			return "", errors.New("add at least one station id")
		}
		fields["stations"] = stations
	} else {
		count, err := whole(or(f.count.String(), "1"))
		if err != nil || count < 1 {
			return "", errors.New("count must be a whole number from 1")
		}
		fields["prefix"], fields["count"] = or(f.prefix.String(), "CP"), count
		if password := f.password.String(); password != "" {
			fields["password"] = password
		}
	}
	for key, value := range map[string]string{"vendor": f.vendor.String(), "model": f.model.String(), "csms": f.csms.String()} {
		if text := strings.TrimSpace(value); text != "" {
			fields[key] = text
		}
	}
	if f.current.index == 1 {
		fields["current"] = f.current.options[f.current.index]
	}
	if f.connect.index == 1 {
		fields["connect"] = false
	}
	numbers := []struct {
		key, label, text string
		isWhole          bool
	}{
		{"phases", "phases", f.phases.String(), true},
		{"max_current_a", "max A", f.maxCurrent.String(), false},
		{"connectors", "connectors", f.connectors.String(), true},
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
	return command.Encode(fields), nil
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
