package ui

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
)

const (
	detailsTab = iota
	configTab
)

type details struct {
	station   string
	connector int
	described description
	tab       int
	config    configView
}

func (m Model) openDetails() Model {
	r, ok := m.selected()
	if !ok {
		return m
	}
	id := m.fleet.Stations[r.station].ID
	m.details = &details{station: id, connector: max(r.connector, 0)}
	return m.sendLine(describeLine(id))
}

func (m Model) describe(d description) Model {
	if m.details == nil || m.details.station != d.Station {
		return m
	}
	next := *m.details
	next.described = d
	m.details = &next
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

func (m Model) detailsKey(msg tea.KeyPressMsg) Model {
	key := msg.String()
	s, ok := m.detailed()
	if !ok {
		m.details = nil
		return m
	}
	if m.details.tab == configTab {
		return m.configKey(msg)
	}
	switch key {
	case "esc", "q":
		m.details = nil
		return m
	case "tab", "shift+tab":
		next := *m.details
		next.tab = configTab
		m.details = &next
		return m
	}
	last := max(len(s.Connectors)-1, 0)
	selected := min(m.details.connector, last)
	next := *m.details
	switch key {
	case "j", "down":
		next.connector = min(selected+1, last)
		m.details = &next
	case "k", "up":
		next.connector = max(selected-1, 0)
		m.details = &next
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
