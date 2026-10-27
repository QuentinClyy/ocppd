package ui

import "encoding/json"

type description struct {
	Kind            string
	Station         string
	Vendor          string
	Model           string
	FirmwareVersion string `json:"firmware_version"`
	Csms            string
}

func parseDescription(line string) (description, bool) {
	var parsed description
	if json.Unmarshal([]byte(line), &parsed) != nil || parsed.Kind != "station_described" {
		return description{}, false
	}
	return parsed, true
}
