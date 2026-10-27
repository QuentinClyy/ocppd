package ui

import "github.com/QuentinClyy/ocppd/ocppd-tui/fleet"

type details struct {
	station   string
	connector int
}

func (m Model) openDetails() Model {
	r, ok := m.selected()
	if !ok {
		return m
	}
	m.details = &details{station: m.fleet.Stations[r.station].ID, connector: max(r.connector, 0)}
	return m
}

func (m Model) detailed() (fleet.Station, bool) {
	for _, s := range m.fleet.Stations {
		if s.ID == m.details.station {
			return s, true
		}
	}
	return fleet.Station{}, false
}

func (m Model) detailsKey(key string) Model {
	s, ok := m.detailed()
	if !ok || key == "esc" || key == "q" || key == "space" {
		m.details = nil
		return m
	}
	last := max(len(s.Connectors)-1, 0)
	selected := min(m.details.connector, last)
	switch key {
	case "j", "down":
		m.details = &details{station: s.ID, connector: min(selected+1, last)}
	case "k", "up":
		m.details = &details{station: s.ID, connector: max(selected-1, 0)}
	case "a":
		if len(s.Connectors) > 0 {
			m.prompt = &prompt{at: target{station: s.ID, connector: s.Connectors[selected].ID}}
		}
	case "p", "s", "u", "f", "c", "x":
		at := target{station: s.ID}
		if len(s.Connectors) > 0 {
			at.connector = s.Connectors[selected].ID
		}
		if line, ok := command(key, at); ok {
			return m.sendLine(line)
		}
	}
	return m
}
