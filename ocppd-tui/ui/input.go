package ui

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/popup"
)

const wheelStep = 3

func (m Model) key(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	key := msg.String()
	switch {
	case key == "ctrl+c":
		return m, tea.Quit
	case m.prompt != nil:
		return m.promptKey(msg), nil
	case m.filtering:
		return m.filterKey(msg), nil
	case m.form != nil:
		return m.formKey(msg), nil
	case m.confirm != nil:
		return m.confirmKey(key), nil
	case m.help:
		return m.helpKey(key), nil
	case m.details != nil:
		return m.detailsKey(msg), nil
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
		m.form = popup.NewSpawn()
	case "Q":
		m.confirm = stopping()
	case "tab":
		m.byStation = !m.byStation
	case "ctrl+u":
		m = m.scrollLog(m.logPage())
	case "ctrl+d":
		m = m.scrollLog(-m.logPage())
	case "/":
		m.filtering = true
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
			m.confirm = removal(at.Station)
		}
	case "a":
		at, ok := m.target()
		if ok && at.Connector == 0 {
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
		line, ok := command.Driver(key, at)
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

func (m Model) filterKey(msg tea.KeyPressMsg) Model {
	switch msg.String() {
	case "enter":
		m.filtering = false
	case "esc":
		m.filters[m.focus], m.filtering = draw.Editor{}, false
	default:
		m.filters[m.focus] = m.filters[m.focus].Edit(msg)
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
	if edited.value.String() == "" {
		return m
	}
	return m.sendLine(command.Authorize(edited.at, edited.value.String()))
}

func (m Model) formKey(msg tea.KeyPressMsg) Model {
	line, closed := m.form.Key(msg)
	if !closed {
		return m
	}
	m.form = nil
	if line == "" {
		return m
	}
	return m.sendLine(line)
}
