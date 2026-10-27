package ui

import tea "charm.land/bubbletea/v2"

const wheelStep = 3

func (m Model) key(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	key := msg.String()
	switch {
	case key == "ctrl+c":
		return m, tea.Quit
	case m.prompt != nil:
		return m.promptKey(msg), nil
	case m.form != nil:
		return m.formKey(msg), nil
	case m.confirm != nil:
		return m.confirmKey(key), nil
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
	case "Q":
		m.confirm = stopping()
	case "d":
		if at, ok := m.target(); ok {
			m.confirm = removal(at.station)
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

func (m Model) wheel(msg tea.MouseWheelMsg) Model {
	var older int
	switch msg.Button {
	case tea.MouseWheelUp:
		older = wheelStep
	case tea.MouseWheelDown:
		older = -wheelStep
	default:
		return m
	}
	l := m.layout()
	overLog := msg.Y >= l.logY-1
	if l.wide {
		overLog = msg.X >= l.split
	}
	if overLog {
		return m.scrollLog(older)
	}
	return m.scrollFleet(-older)
}
