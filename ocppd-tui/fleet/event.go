package fleet

import (
	"encoding/json"
	"fmt"
)

type event struct {
	Station       string  `json:"station"`
	Kind          string  `json:"kind"`
	At            int64   `json:"at"`
	Status        string  `json:"status"`
	Connector     int     `json:"connector"`
	Evse          int     `json:"evse"`
	TransactionID any     `json:"transaction_id"`
	EventType     string  `json:"event_type"`
	PowerW        float64 `json:"power_w"`
	RegisterWh    float64 `json:"register_wh"`
	Soc           float64 `json:"soc"`
}

func parseEvent(raw string) (event, bool) {
	var parsed event
	if err := json.Unmarshal([]byte(raw), &parsed); err != nil || parsed.Station == "" {
		return event{}, false
	}
	return parsed, true
}

func (e event) slot() int {
	if e.Evse > 0 {
		return e.Evse
	}
	return e.Connector
}

func (e event) transaction() string {
	if e.TransactionID == nil {
		return ""
	}
	if id, ok := e.TransactionID.(float64); ok {
		return fmt.Sprintf("%.0f", id)
	}
	return fmt.Sprint(e.TransactionID)
}
