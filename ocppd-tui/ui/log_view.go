package ui

import (
	"charm.land/lipgloss/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

func (m Model) logPane(a area) []string {
	height := max(a.height-1, 0)
	logs := m.logLines()
	offset := m.clampLog(m.logOffset)
	top := max(len(logs)-height-offset, 0)
	shown := draw.TailLines(logs, height, offset)
	for index, line := range shown {
		if style, ok := m.logStyle(len(logs) - 1 - top - index); ok {
			shown[index] = style.Render(draw.Pad(line, a.width-1))
		}
	}
	content := draw.Scrollbar(draw.Fill(shown, height), a.width, len(logs), top, " ")
	return append([]string{m.logTitle()}, content...)
}

func (m Model) logStyle(line int) (lipgloss.Style, bool) {
	newest, oldest := m.selection()
	switch {
	case m.focus != logPane && line == m.logCursor:
		return draw.Idle, true
	case m.focus != logPane:
		return draw.Plain, false
	case m.visual && line == m.logCursor:
		return draw.Chosen, true
	case line >= newest && line <= oldest:
		return draw.Selected, true
	}
	return draw.Plain, false
}

func (m Model) logTitle() string {
	station := m.logStation()
	if station == "" {
		station = "all stations"
	}
	filter := ""
	if text := m.filters[logPane].String(); text != "" {
		filter = draw.Dim.Render(" · /" + text)
	}
	if m.focus != logPane {
		return draw.Dim.Render(" LOGS · "+station) + filter
	}
	return " LOGS · " + draw.Bold.Render(station) + filter
}
