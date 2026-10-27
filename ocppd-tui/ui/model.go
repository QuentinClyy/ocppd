package ui

import (
	"time"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ocppd"
)

type Model struct {
	csms        string
	expected    string
	daemon      string
	send        func(string) error
	zone        *time.Location
	log         []entry
	fleet       fleet.Fleet
	expanded    map[string]bool
	cursor      int
	fleetScroll int
	pendingG    bool
	byStation   bool
	logOffset   int
	help        bool
	helpTop     int
	focus       pane
	zoomed      bool
	dragging    bool
	sideWidth   int
	bottomRows  int
	prompt      *prompt
	form        *form
	confirm     *confirmation
	stopping    bool
	lost        bool
	width       int
	height      int
}

func New(send func(string) error) Model {
	return Model{send: send, zone: time.Local, expanded: map[string]bool{}}
}

func (m Model) Expecting(version string) Model {
	m.expected = version
	return m
}

func (m Model) Lost() bool {
	return m.lost
}

func (m Model) Init() tea.Cmd {
	return nil
}

func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width, m.height = msg.Width, msg.Height
	case tea.KeyPressMsg:
		next, cmd := m.key(msg)
		m = next.(Model)
		m.fleetScroll = m.fleetTop()
		return m, cmd
	case tea.MouseWheelMsg:
		if !m.overlaid() {
			m = m.wheel(msg)
		}
	case tea.MouseClickMsg:
		if !m.overlaid() {
			m = m.click(msg)
			m.fleetScroll = m.fleetTop()
		}
	case tea.MouseMotionMsg:
		if m.dragging {
			m = m.dragTo(msg.X, msg.Y)
		}
	case tea.MouseReleaseMsg:
		m.dragging = false
	case ocppd.EventMsg:
		if g, ok := parseGreeting(string(msg)); ok {
			m.csms, m.daemon = g.Csms, g.Version
			return m, nil
		}
		station, _ := m.fleet.Apply(string(msg))
		m = m.addLog(entry{station, logLine(string(msg), m.zone)})
	case ocppd.ClosedMsg:
		m.lost = !m.stopping
		return m, tea.Quit
	}
	return m, nil
}

func (m Model) sendLine(line string) Model {
	if err := m.send(line); err != nil {
		m = m.addLog(entry{text: "send: " + err.Error()})
	}
	return m
}

func (m Model) View() tea.View {
	view := tea.NewView(render(m))
	view.AltScreen = true
	view.MouseMode = tea.MouseModeCellMotion
	return view
}

func (m Model) overlaid() bool {
	return m.help || m.form != nil
}

func (m Model) outdated() bool {
	return m.expected != "" && m.daemon != "" && m.daemon != m.expected
}
