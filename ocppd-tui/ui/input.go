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
		return m.helpKey(key), nil
	case m.details != nil:
		return m.detailsKey(key), nil
	}
	wasG := m.pendingG
	m.pendingG = false
	if direction, ok := focusKeys[key]; ok {
		return m.focusToward(direction), nil
	}
	switch key {
	case "q":
		return m, tea.Quit
	case "?":
		m.help = true
	case "n":
		m.form = newForm()
	case "Q":
		m.confirm = stopping()
	case "tab":
		m.byStation = !m.byStation
	case "ctrl+u":
		m = m.scrollLog(m.logPage())
	case "ctrl+d":
		m = m.scrollLog(-m.logPage())
	case "z":
		m.zoomed = !m.zoomed
	case "+":
		m = m.grow(1)
	case "-":
		m = m.grow(-1)
	case "=":
		m.sideWidth, m.bottomRows = 0, 0
	case "g":
		if wasG {
			return m.paneKey("gg"), nil
		}
		m.pendingG = true
	default:
		return m.paneKey(key), nil
	}
	return m, nil
}

func (m Model) paneKey(key string) Model {
	switch m.focus {
	case fleetPane:
		return m.fleetKey(key)
	case logPane:
		return m.logKey(key)
	}
	return m
}

func (m Model) fleetKey(key string) Model {
	switch key {
	case "space":
		return m.openDetails()
	case "d":
		if at, ok := m.target(); ok {
			m.confirm = removal(at.station)
		}
	case "a":
		at, ok := m.target()
		if ok && at.connector == 0 {
			return m.open()
		}
		if ok {
			m.prompt = &prompt{at: at}
		}
	case "p", "s", "u", "f", "c", "x":
		at, ok := m.target()
		if !ok {
			return m
		}
		line, ok := command(key, at)
		if !ok {
			return m.open()
		}
		return m.sendLine(line)
	default:
		return m.move(key)
	}
	return m
}

func (m Model) logKey(key string) Model {
	switch key {
	case "j", "down":
		return m.scrollLog(-1)
	case "k", "up":
		return m.scrollLog(1)
	case "gg":
		return m.scrollLog(len(m.logLines()))
	case "G":
		m.logOffset = 0
	}
	return m
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
	key := msg.String()
	switch {
	case key == "esc" && !m.form.normal:
		m.form.normal = true
		return m
	case key == "esc" || key == "q" && m.form.normal:
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
