package ui

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/popup"
)

const recentLines = 3

func (m Model) openDetails() Model {
	r, ok := m.selected()
	if !ok {
		return m
	}
	opened, describe := popup.OpenStation(m.fleet.Stations[r.station].ID, max(r.connector, 0))
	m.details = &opened
	return m.sendLine(describe)
}

func (m Model) detailed() (fleet.Station, bool) {
	for _, s := range m.fleet.Stations {
		if s.ID == m.details.ID() {
			return s, true
		}
	}
	return fleet.Station{}, false
}

func (m Model) detailsKey(msg tea.KeyPressMsg) Model {
	s, ok := m.detailed()
	if !ok {
		m.details = nil
		return m
	}
	return m.apply(m.details.Key(msg, s))
}

func (m Model) apply(r popup.Result) Model {
	m.details = r.Popup
	for _, line := range r.Send {
		m = m.sendLine(line)
	}
	if r.Prompt != nil {
		m.prompt = &prompt{at: *r.Prompt}
	}
	return m
}

func (m Model) observe(line string) Model {
	if m.details == nil {
		return m
	}
	next, send := m.details.Observe(line)
	return m.apply(popup.Result{Popup: &next, Send: send})
}

func (m Model) detailsFrame() []string {
	s, _ := m.detailed()
	var recent []string
	for index := len(m.log) - 1; index >= 0 && len(recent) < recentLines; index-- {
		if m.log[index].station == s.ID {
			recent = append([]string{m.log[index].withoutStation()}, recent...)
		}
	}
	context := popup.Context{Station: s, Recent: recent, Zone: m.zone}
	return m.modal(m.details.Frame(context, m.modalRows()))
}
