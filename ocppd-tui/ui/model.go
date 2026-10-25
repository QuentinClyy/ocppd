package ui

import (
	"time"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ocppd"
)

type Model struct {
	csms        string
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
	prompt      *prompt
	form        *form
	removing    string
	width       int
	height      int
}

func New(csms string, send func(string) error) Model {
	return Model{csms: csms, send: send, zone: time.Local, expanded: map[string]bool{}}
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
		if !m.help && m.form == nil {
			m = m.wheel(msg)
		}
	case ocppd.EventMsg:
		station, _ := m.fleet.Apply(string(msg))
		m = m.addLog(entry{station, logLine(string(msg), m.zone)})
	case ocppd.ErrorMsg:
		m = m.addLog(entry{text: "stderr    " + string(msg)})
	case ocppd.ExitMsg:
		m = m.addLog(entry{text: exitLine(msg.Err)})
	}
	return m, nil
}

func (m Model) key(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	key := msg.String()
	switch {
	case key == "ctrl+c":
		return m, tea.Quit
	case m.prompt != nil:
		return m.promptKey(msg), nil
	case m.form != nil:
		return m.formKey(msg), nil
	case m.removing != "":
		return m.removeKey(key), nil
	case m.help:
		m.help = key != "esc" && key != "?" && key != "q"
		return m, nil
	}
	wasG := m.pendingG
	m.pendingG = false
	switch key {
	case "q":
		return m, tea.Quit
	case "?":
		m.help = true
	case "n":
		m.form = newForm()
	case "d":
		if at, ok := m.target(); ok {
			m.removing = at.station
		}
	case "tab":
		m.byStation = !m.byStation
	case "ctrl+u":
		m = m.scrollLog(m.logPage())
	case "ctrl+d":
		m = m.scrollLog(-m.logPage())
	case "g":
		if wasG {
			return m.move("g"), nil
		}
		m.pendingG = true
	case "a":
		at, ok := m.target()
		if ok && at.connector == 0 {
			return m.move("l"), nil
		}
		if ok {
			m.prompt = &prompt{at: at}
		}
	case "p", "s", "u", "f", "c", "x":
		at, ok := m.target()
		if !ok {
			return m, nil
		}
		line, ok := command(key, at)
		if !ok {
			return m.move("l"), nil
		}
		return m.sendLine(line), nil
	default:
		return m.move(key), nil
	}
	return m, nil
}

func (m Model) promptKey(msg tea.KeyPressMsg) Model {
	if msg.String() == "esc" {
		m.prompt = nil
		return m
	}
	edited, done := m.prompt.edit(msg)
	if !done {
		m.prompt = &edited
		return m
	}
	m.prompt = nil
	if len(edited.value) == 0 {
		return m
	}
	return m.sendLine(authorize(edited.at, string(edited.value)))
}

func (m Model) formKey(msg tea.KeyPressMsg) Model {
	if msg.String() == "esc" {
		m.form = nil
		return m
	}
	if !m.form.edit(msg) {
		return m
	}
	line, err := m.form.line()
	if err != nil {
		m.form.err = err.Error()
		return m
	}
	m.form = nil
	return m.sendLine(line)
}

func (m Model) removeKey(key string) Model {
	station := m.removing
	m.removing = ""
	if key != "y" && key != "d" {
		return m
	}
	m.fleet.Remove(station)
	delete(m.expanded, station)
	return m.sendLine(encode(map[string]any{"command": "remove", "station": station}))
}

func (m Model) sendLine(line string) Model {
	if err := m.send(line); err != nil {
		m = m.addLog(entry{text: "send: " + err.Error()})
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

func (m Model) logPage() int {
	return max(m.height/4, 1)
}

func (m Model) View() tea.View {
	view := tea.NewView(render(m))
	view.AltScreen = true
	view.MouseMode = tea.MouseModeCellMotion
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
