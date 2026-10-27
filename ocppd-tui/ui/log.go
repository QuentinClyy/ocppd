package ui

import (
	"encoding/json"
	"fmt"
	"maps"
	"slices"
	"strings"
	"time"
)

const logLimit = 10000

type entry struct {
	station string
	text    string
}

func logLine(raw string, zone *time.Location) string {
	var event map[string]any
	if err := json.Unmarshal([]byte(raw), &event); err != nil {
		return raw
	}
	at, _ := event["at"].(float64)
	station, _ := event["station"].(string)
	kind, _ := event["kind"].(string)
	var fields []string
	for _, key := range slices.Sorted(maps.Keys(event)) {
		if key != "at" && key != "station" && key != "kind" {
			fields = append(fields, key+"="+text(event[key]))
		}
	}
	clock := time.UnixMilli(int64(at)).In(zone).Format("15:04:05")
	return fmt.Sprintf("%s  %-10s %-22s %s", clock, station, kind, strings.Join(fields, " "))
}

func text(value any) string {
	if s, ok := value.(string); ok {
		return s
	}
	encoded, err := json.Marshal(value)
	if err != nil {
		return fmt.Sprint(value)
	}
	return string(encoded)
}

func appendLog(log []entry, line entry) []entry {
	log = append(log, line)
	if len(log) > logLimit {
		return log[len(log)-logLimit:]
	}
	return log
}

func filtered(log []entry, station string) []string {
	var lines []string
	for _, line := range log {
		if station == "" || line.station == station {
			lines = append(lines, line.text)
		}
	}
	return lines
}

func (m Model) addLog(line entry) Model {
	m.log = appendLog(m.log, line)
	if m.logOffset > 0 && (m.logStation() == "" || m.logStation() == line.station) {
		m.logOffset++
	}
	return m
}

func (m Model) scrollLog(lines int) Model {
	m.logOffset = m.clampLog(m.clampLog(m.logOffset) + lines)
	return m
}

func (m Model) clampLog(offset int) int {
	return min(max(offset, 0), max(len(m.logLines())-m.paneRows(logPane), 0))
}

func (m Model) logPage() int {
	return max(m.height/4, 1)
}

func (m Model) logStation() string {
	r, ok := m.selected()
	if !m.byStation || !ok {
		return ""
	}
	return m.fleet.Stations[r.station].ID
}

func (m Model) logLines() []string {
	return filtered(m.log, m.logStation())
}
