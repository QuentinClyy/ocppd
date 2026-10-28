package ui

import (
	"strings"

	tea "charm.land/bubbletea/v2"
)

func (m Model) clampLog(offset int) int {
	return min(max(offset, 0), max(len(m.logLines())-m.paneRows(logPane), 0))
}

func (m Model) moveLog(cursor int) Model {
	m.logCursor = min(max(cursor, 0), max(len(m.logLines())-1, 0))
	offset := min(m.clampLog(m.logOffset), m.logCursor)
	m.logOffset = m.clampLog(max(offset, m.logCursor-m.paneRows(logPane)+1))
	return m
}

func (m Model) scrollLog(lines int) Model {
	m.logOffset = m.clampLog(m.clampLog(m.logOffset) + lines)
	bottom := m.logOffset + max(m.paneRows(logPane), 1) - 1
	m.logCursor = min(max(m.logCursor, m.logOffset), bottom, max(len(m.logLines())-1, 0))
	return m
}

func (m Model) pageLog(lines int) Model {
	cursor := m.logCursor + lines
	return m.scrollLog(lines).moveLog(cursor)
}

func (m Model) logLine(number int) Model {
	return m.moveLog(len(m.logLines()) - number)
}

func (m Model) resetLog() Model {
	m.logCursor, m.logOffset, m.visual = 0, 0, false
	return m
}

func (m Model) selection() (int, int) {
	if !m.visual {
		return m.logCursor, m.logCursor
	}
	return min(m.logAnchor, m.logCursor), max(m.logAnchor, m.logCursor)
}

func (m Model) yank(newest, oldest int) (Model, tea.Cmd) {
	lines := m.logLines()
	newest, oldest = max(newest, 0), min(oldest, len(lines)-1)
	if newest > oldest {
		return m, nil
	}
	m.visual, m.yanked = false, oldest-newest+1
	return m, tea.SetClipboard(strings.Join(lines[len(lines)-1-oldest:len(lines)-newest], "\n"))
}
