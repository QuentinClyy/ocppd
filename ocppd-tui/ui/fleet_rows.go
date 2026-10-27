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
	case "gg":
		m.cursor = 0
	case "G":
		m.cursor = len(visible) - 1
	case "enter":
		if current.connector != stationRow {
			m.cursor -= current.connector + 1
			delete(m.expanded, s.ID)
		} else {
			m.expanded[s.ID] = !m.expanded[s.ID]
		}
	}
	return m
}

func (m Model) open() Model {
	r, ok := m.selected()
	if !ok || r.connector != stationRow {
		return m
	}
	s := m.fleet.Stations[r.station]
	if m.expanded[s.ID] && len(s.Connectors) > 0 {
		m.cursor = min(m.cursor, len(rows(m.fleet, m.expanded))-1) + 1
	} else {
		m.expanded[s.ID] = true
	}
	return m
}

func (m Model) target() (target, bool) {
	r, ok := m.selected()
	if !ok {
		return target{}, false
	}
	s := m.fleet.Stations[r.station]
	at := target{station: s.ID}
	switch {
	case r.connector != stationRow:
		at.connector = s.Connectors[r.connector].ID
	case len(s.Connectors) == 1:
		at.connector = s.Connectors[0].ID
	}
	return at, true
}

func (m Model) fleetTop() int {
	total := len(rows(m.fleet, m.expanded))
	return follow(m.fleetScroll, min(m.cursor, max(total-1, 0)), m.paneRows(fleetPane), total)
}

func (m Model) scrollFleet(lines int) Model {
	total := len(rows(m.fleet, m.expanded))
	height := m.paneRows(fleetPane)
	if total == 0 || height == 0 {
		return m
	}
	top := min(max(m.fleetTop()+lines, 0), max(total-height, 0))
	m.cursor = min(max(min(m.cursor, total-1), top), top+height-1)
	m.fleetScroll = top
	return m
}
