package ui

func (m Model) logPane(a area) []string {
	height := max(a.height-1, 0)
	logs := m.logLines()
	offset := m.clampLog(m.logOffset)
	top := max(len(logs)-height-offset, 0)
	content := scrollbar(fill(tailLines(logs, height, offset), height), a.width, len(logs), top, " ")
	return append([]string{m.logTitle()}, content...)
}

func (m Model) logTitle() string {
	station := m.logStation()
	if station == "" {
		station = "all stations"
	}
	if m.focus != logPane {
		return dim.Render(" " + station + " · log")
	}
	return " " + bold.Render(station) + dim.Render(" · log")
}
