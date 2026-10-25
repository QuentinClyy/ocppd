package ui

import (
	"time"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ocppd"
)

type Model struct {
	csms      string
	zone      *time.Location
	log       []entry
	fleet     fleet.Fleet
	collapsed map[string]bool
	cursor    int
	pendingG  bool
	byStation bool
	width     int
	height    int
}

func New(csms string) Model {
	return Model{csms: csms, zone: time.Local, collapsed: map[string]bool{}}
}

func (m Model) Init() tea.Cmd {
	return nil
}

func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width, m.height = msg.Width, msg.Height
	case tea.KeyPressMsg:
		return m.key(msg.String())
	case ocppd.EventMsg:
		station, _ := m.fleet.Apply(string(msg))
		m.log = appendLog(m.log, entry{station, logLine(string(msg), m.zone)})
	case ocppd.ErrorMsg:
		m.log = appendLog(m.log, entry{text: "stderr    " + string(msg)})
	case ocppd.ExitMsg:
		m.log = appendLog(m.log, entry{text: exitLine(msg.Err)})
	}
	return m, nil
}

func (m Model) key(key string) (tea.Model, tea.Cmd) {
	wasG := m.pendingG
	m.pendingG = false
	switch key {
	case "q", "ctrl+c":
		return m, tea.Quit
	case "tab":
		m.byStation = !m.byStation
	case "g":
		if !wasG {
			m.pendingG = true
			return m, nil
		}
		return m.move("g"), nil
	default:
		return m.move(key), nil
	}
	return m, nil
}

func (m Model) View() tea.View {
	view := tea.NewView(render(m))
	view.AltScreen = true
	return view
}

func (m Model) logStation() string {
	r, ok := m.selected()
	if !m.byStation || !ok {
		return ""
	}
	return m.fleet.Stations[r.station].ID
}

func exitLine(err error) string {
	if err != nil {
		return "ocppd exited: " + err.Error()
	}
	return "ocppd exited"
}
