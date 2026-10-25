package ui

import "github.com/QuentinClyy/ocppd/ocppd-tui/fleet"

const stationRow = -1

type row struct {
	station   int
	connector int
}

func rows(f fleet.Fleet, expanded map[string]bool) []row {
	var visible []row
	for index, s := range f.Stations {
		visible = append(visible, row{index, stationRow})
		if !expanded[s.ID] {
			continue
		}
		for connector := range s.Connectors {
			visible = append(visible, row{index, connector})
		}
	}
	return visible
}

func (m Model) selected() (row, bool) {
	visible := rows(m.fleet, m.expanded)
	if len(visible) == 0 {
		return row{}, false
	}
	return visible[min(m.cursor, len(visible)-1)], true
}

func (m Model) move(key string) Model {
	visible := rows(m.fleet, m.expanded)
	if len(visible) == 0 {
		return m
	}
	m.cursor = min(m.cursor, len(visible)-1)
	current := visible[m.cursor]
	s := m.fleet.Stations[current.station]
	switch key {
	case "j", "down":
		m.cursor = min(m.cursor+1, len(visible)-1)
	case "k", "up":
		m.cursor = max(m.cursor-1, 0)
	case "g":
		m.cursor = 0
	case "G":
		m.cursor = len(visible) - 1
	case "l", "right", "enter":
		if current.connector != stationRow {
			return m
		}
		if m.expanded[s.ID] && len(s.Connectors) > 0 {
			m.cursor++
		} else {
			m.expanded[s.ID] = true
		}
	case "h", "left":
		if current.connector != stationRow {
			m.cursor -= current.connector + 1
		} else {
			delete(m.expanded, s.ID)
		}
	}
	return m
}
