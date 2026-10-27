package ui

import "github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"

func (m Model) logPane(a area) []string {
	height := max(a.height-1, 0)
	logs := m.logLines()
	offset := m.clampLog(m.logOffset)
	top := max(len(logs)-height-offset, 0)
	content := draw.Scrollbar(draw.Fill(draw.TailLines(logs, height, offset), height), a.width, len(logs), top, " ")
	return append([]string{m.logTitle()}, content...)
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
