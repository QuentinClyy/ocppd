package ui

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/motion"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/popup"
)

const wheelStep = 3

func (m Model) key(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	key := msg.String()
	m.yanked = 0
	switch {
	case key == "ctrl+c":
		return m, tea.Quit
	case m.prompt != nil:
		return m.promptKey(msg), nil
	case m.filtering:
		return m.filterKey(msg), nil
	case m.help:
		return m.helpKey(key), nil
	case m.opensPopupHelp(key):
		m.help = true
		return m, nil
	case m.form != nil:
		return m.formKey(msg), nil
	case m.confirm != nil:
		return m.confirmKey(key), nil
	case m.details != nil:
		return m.detailsKey(msg), nil
	}
	prefixes := "gy"
	if m.visual {
		prefixes = "g"
	}
	pending, count, key := motion.Feed(m.pending, key, prefixes)
	m.pending = pending
	if direction, ok := focusKeys[key]; ok {
		return m.focusToward(direction), nil
	}
	switch key {
	case "":
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
		m = m.resetLog()
	case "ctrl+u":
		m = m.pageLog(m.logPage())
	case "ctrl+d":
		m = m.pageLog(-m.logPage())
	case "/":
		m.filtering = true
	case "z":
		m.zoomed = !m.zoomed
	case "+":
		m = m.grow(max(count, 1))
	case "-":
		m = m.grow(-max(count, 1))
	case "=":
		m.sideWidth, m.bottomRows = 0, 0
	default:
		return m.paneKey(key, count)
	}
	return m, nil
}

func (m Model) paneKey(key string, count int) (Model, tea.Cmd) {
	switch m.focus {
	case fleetPane:
		return m.fleetKey(key, count), nil
	case logPane:
		return m.logKey(key, count)
	}
	return m, nil
}

func (m Model) fleetKey(key string, count int) Model {
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
		return m.move(key, count)
	}
	return m
}

func (m Model) logKey(key string, count int) (Model, tea.Cmd) {
	last := len(m.logLines()) - 1
	if index, ok := motion.Move(key, count, last-m.logCursor, last); ok {
		return m.moveLog(last - index), nil
	}
	switch key {
	case "V", "v":
		m.visual, m.logAnchor = !m.visual, m.logCursor
	case "esc":
		m.visual = false
	case "y":
		if m.visual {
			return m.yank(m.selection())
		}
	case "yy", "Y":
		return m.yank(m.logCursor-max(count, 1)+1, m.logCursor)
	}
	return m, nil
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
	if m.focus == logPane {
		return m.resetLog()
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
