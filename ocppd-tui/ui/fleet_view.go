package ui

import (
	"fmt"
	"strings"

	"charm.land/lipgloss/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

var columns = []int{7, 6, 10, 13, 7, 6, 6}

type cell struct {
	text  string
	style lipgloss.Style
}

func fleetHeader(nameWidth int) string {
	names := []string{"  STATION", "OCPP", "LINK", "BOOT", "STATUS", "kW", "SOC", "TX"}
	var line strings.Builder
	for index, name := range names {
		line.WriteString(draw.Pad(name, width(index, nameWidth)))
	}
	return line.String()
}

func fleetLines(f fleet.Fleet, visible []row, expanded map[string]bool, cursor, nameWidth int) []string {
	lines := make([]string, len(visible))
	for index, r := range visible {
		s := f.Stations[r.station]
		cells := stationCells(s, expanded[s.ID])
		if r.connector != stationRow {
			cells = connectorCells(s, r.connector)
		}
		lines[index] = renderCells(cells, nameWidth, index == cursor)
	}
	return lines
}

func stationCells(s fleet.Station, isExpanded bool) []cell {
	marker := "▸ "
	if isExpanded {
		marker = "▾ "
	}
	if len(s.Connectors) == 0 {
		marker = "  "
	}
	status, style := summary(s)
	return []cell{
		{marker + s.ID, draw.Plain},
		{s.Ocpp, draw.Plain},
		{draw.Dash(s.Link), draw.StatusStyle(s.Link)},
		{draw.Dash(s.Boot), draw.StatusStyle(s.Boot)},
		{status, style},
		{totalKW(s), draw.Plain},
	}
}

func connectorCells(s fleet.Station, index int) []cell {
	c := s.Connectors[index]
	branch := "  ├─ "
	if index == len(s.Connectors)-1 {
		branch = "  └─ "
	}
	kw, soc := "—", "—"
	if c.Sampled {
		kw, soc = fmt.Sprintf("%.1f", c.PowerW/1000), fmt.Sprintf("%.0f%%", c.Soc*100)
	}
	return []cell{
		{branch + fmt.Sprint(c.ID), draw.Dim},
		{"", draw.Plain},
		{"", draw.Plain},
		{"", draw.Plain},
		{draw.Dash(c.Status), draw.StatusStyle(c.Status)},
		{kw, draw.Plain},
		{soc, draw.Plain},
		{draw.Tail(c.Transaction, columns[6]-1), draw.Plain},
	}
}

func renderCells(cells []cell, nameWidth int, isSelected bool) string {
	var line strings.Builder
	for index, c := range cells {
		text := draw.Pad(c.text, width(index, nameWidth))
		if isSelected {
			line.WriteString(text)
		} else {
			line.WriteString(c.style.Render(text))
		}
	}
	if isSelected {
		return draw.Selected.Render(draw.Pad(line.String(), fleetWidth(nameWidth)))
	}
	return strings.TrimRight(line.String(), " ")
}

func summary(s fleet.Station) (string, lipgloss.Style) {
	var faulted, charging, reserved int
	for _, c := range s.Connectors {
		switch {
		case c.Status == "Faulted":
			faulted++
		case c.Sampled && c.PowerW > 0:
			charging++
		case c.Status == "Reserved":
			reserved++
		}
	}
	switch {
	case faulted > 0:
		return fmt.Sprintf("%d faulted", faulted), draw.Warn
	case charging > 0:
		return fmt.Sprintf("%d charging", charging), draw.Good
	case reserved > 0:
		return fmt.Sprintf("%d reserved", reserved), draw.Accent
	}
	return "", draw.Plain
}

func totalKW(s fleet.Station) string {
	total, sampled := 0.0, false
	for _, c := range s.Connectors {
		if c.Sampled {
			total, sampled = total+c.PowerW, true
		}
	}
	if !sampled {
		return "—"
	}
	return fmt.Sprintf("%.1f", total/1000)
}

func width(index, nameWidth int) int {
	if index == 0 {
		return nameWidth
	}
	return columns[index-1]
}

func fleetWidth(nameWidth int) int {
	total := nameWidth
	for _, column := range columns {
		total += column
	}
	return total
}
