package ui

func (m Model) logPane(l layout, width int) []string {
	logs := m.logLines()
	offset := m.clampLog(m.logOffset)
	top := max(len(logs)-l.logRows-offset, 0)
	return scrollbar(fill(tailLines(logs, l.logRows, offset), l.logRows), width, len(logs), top, " ")
}

func (m Model) logTitle() string {
	if station := m.logStation(); station != "" {
		return " " + bold.Render(station) + dim.Render(" · log")
	}
	return " " + bold.Render("all stations") + dim.Render(" · log")
}
