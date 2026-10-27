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
	clock   string
	text    string
}

func (e entry) line() string {
	if e.clock == "" {
		return e.text
	}
	return fmt.Sprintf("%s  %-10s %s", e.clock, e.station, e.text)
}

func (e entry) withoutStation() string {
	if e.clock == "" {
		return e.text
	}
	return e.clock + "  " + e.text
}

func logEntry(raw string, zone *time.Location) entry {
	var event map[string]any
	if err := json.Unmarshal([]byte(raw), &event); err != nil {
		return entry{text: raw}
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
	return entry{station: station, clock: clock, text: fmt.Sprintf("%-22s %s", kind, strings.Join(fields, " "))}
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

func (m Model) addLog(line entry) Model {
	m.log = appendLog(m.log, line)
	if m.logOffset > 0 && m.showsLog(line) {
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

func (m Model) showsLog(e entry) bool {
	station := m.logStation()
	return (station == "" || e.station == station) && matches(e.line(), m.filters[logPane].String())
}

func (m Model) logLines() []string {
	var lines []string
	for _, e := range m.log {
		if m.showsLog(e) {
			lines = append(lines, e.line())
		}
	}
	return lines
}
