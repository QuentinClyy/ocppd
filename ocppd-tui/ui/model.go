package ui

import (
	"time"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ocppd"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/popup"
)

type form interface {
	Key(tea.KeyPressMsg) (string, bool)
	Paste(string)
	Pastes(string) bool
	Typing() bool
	Frame(int) draw.Frame
	Bindings() []draw.Binding
	Bar(int) string
}

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
	pending     string
	byStation   bool
	logOffset   int
	logCursor   int
	logAnchor   int
	visual      bool
	yanked      int
	help        bool
	helpTop     int
	focus       pane
	filters     [paneCount]draw.Editor
	filtering   bool
	zoomed      bool
	dragging    bool
	sel         selection
	sideWidth   int
	bottomRows  int
	prompt      *prompt
	form        form
	details     *popup.Station
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
	case tea.PasteMsg:
		m = m.paste(msg.Content)
	case tea.ClipboardMsg:
		m = m.paste(msg.Content)
	case tea.MouseWheelMsg:
		if !m.overlaid() {
			m = m.wheel(msg)
		}
	case tea.MouseClickMsg:
		m = m.press(msg)
	case tea.MouseMotionMsg:
		m = m.motion(msg)
	case tea.MouseReleaseMsg:
		return m.lift()
	case ocppd.EventMsg:
		if g, ok := parseGreeting(string(msg)); ok {
			m.csms, m.daemon = g.Csms, g.Version
			return m, nil
		}
		m = m.observe(string(msg))
		if popup.IsDescription(string(msg)) {
			return m, nil
		}
		m.fleet.Apply(string(msg))
		m = m.addLog(logEntry(string(msg), m.zone))
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
	return m.help || m.form != nil || m.details != nil
}

func (m Model) outdated() bool {
	return m.expected != "" && m.daemon != "" && m.daemon != m.expected
}
