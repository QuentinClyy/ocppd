package main

import (
	"encoding/json"
	"fmt"
	"maps"
	"slices"
	"strings"
	"time"
)

const logLimit = 10000

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

func appendLog(log []string, line string) []string {
	log = append(log, line)
	if len(log) > logLimit {
		return log[len(log)-logLimit:]
	}
	return log
}
