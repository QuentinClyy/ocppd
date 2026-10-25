package main

import (
	"strings"
	"time"

	tea "charm.land/bubbletea/v2"
	"github.com/charmbracelet/x/ansi"
)

type model struct {
	csms   string
	zone   *time.Location
	log    []string
	width  int
	height int
}

func newModel(csms string) model {
	return model{csms: csms, zone: time.Local}
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width, m.height = msg.Width, msg.Height
	case tea.KeyPressMsg:
		if msg.String() == "q" || msg.String() == "ctrl+c" {
			return m, tea.Quit
		}
	case eventMsg:
		m.log = appendLog(m.log, logLine(string(msg), m.zone))
	case errorMsg:
		m.log = appendLog(m.log, "stderr    "+string(msg))
	case exitMsg:
		m.log = appendLog(m.log, exitLine(msg.err))
	}
	return m, nil
}

func (m model) View() tea.View {
	view := tea.NewView(render(m))
	view.AltScreen = true
	return view
}

func exitLine(err error) string {
	if err != nil {
		return "ocppd exited: " + err.Error()
	}
	return "ocppd exited"
}

func render(m model) string {
	rule := " " + strings.Repeat("─", max(m.width-2, 0))
	rows := max(m.height-4, 0)
	shown := m.log[max(len(m.log)-rows, 0):]
	lines := []string{" ocppd  " + m.csms, rule}
	for _, line := range shown {
		lines = append(lines, ansi.Truncate(" "+line, m.width, ""))
	}
	for range rows - len(shown) {
		lines = append(lines, "")
	}
	return strings.Join(append(lines, rule, " q quit"), "\n")
}
