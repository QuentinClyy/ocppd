package ui

import (
	tea "charm.land/bubbletea/v2"
	"github.com/charmbracelet/x/ansi"
)

const wheelStep = 3

func follow(top, cursor, rows, total int) int {
	top = min(max(top, 0), max(total-rows, 0))
	if cursor < top {
		top = cursor
	}
	if cursor >= top+rows {
		top = cursor - rows + 1
	}
	return max(top, 0)
}

func (m Model) fleetTop() int {
	total := len(rows(m.fleet, m.expanded))
	return follow(m.fleetScroll, min(m.cursor, max(total-1, 0)), m.layout().fleetRows, total)
}

func (m Model) scrollFleet(lines int) Model {
	total := len(rows(m.fleet, m.expanded))
	height := m.layout().fleetRows
	if total == 0 || height == 0 {
		return m
	}
	top := min(max(m.fleetTop()+lines, 0), max(total-height, 0))
	m.cursor = min(max(min(m.cursor, total-1), top), top+height-1)
	m.fleetScroll = top
	return m
}

func (m Model) clampLog(offset int) int {
	return min(max(offset, 0), max(len(m.logLines())-m.layout().logRows, 0))
}

func (m Model) scrollLog(lines int) Model {
	m.logOffset = m.clampLog(m.clampLog(m.logOffset) + lines)
	return m
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

func (m Model) addLog(line entry) Model {
	m.log = appendLog(m.log, line)
	if m.logOffset > 0 && (m.logStation() == "" || m.logStation() == line.station) {
		m.logOffset++
	}
	return m
}

func thumb(total, rows, top int) (int, int) {
	if total <= rows || rows == 0 {
		return 0, 0
	}
	size := max(rows*rows/total, 1)
	start := top * (rows - size) / (total - rows)
	return start, start + size
}

func scrollbar(lines []string, width, total, top int, track string) []string {
	start, end := thumb(total, len(lines), top)
	for index, line := range lines {
		mark := track
		if index >= start && index < end {
			mark = accent.Render("▐")
		}
		lines[index] = pad(ansi.Truncate(line, width-1, ""), width-1) + mark
	}
	return lines
}
