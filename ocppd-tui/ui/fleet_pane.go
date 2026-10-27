package ui

import (
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/charmbracelet/x/ansi"
)

func (m Model) fleetPane(a area) []string {
	nameWidth := narrowName
	track := " "
	if a.x+a.width < m.width {
		nameWidth, track = wideName, draw.Dim.Render("│")
	}
	visible := rows(m.fleet, m.expanded)
	cursor := min(m.cursor, max(len(visible)-1, 0))
	lines := fleetLines(m.fleet, visible, m.expanded, cursor, nameWidth)
	if m.focus != fleetPane && cursor < len(lines) {
		lines[cursor] = draw.Idle.Render(draw.Pad(ansi.Strip(lines[cursor]), fleetWidth(nameWidth)))
	}
	height := max(a.height-1, 0)
	content := make([]string, height)
	for index, line := range draw.Window(lines, m.fleetTop(), height) {
		content[index] = " " + line
	}
	content = draw.Scrollbar(content, a.width, len(visible), m.fleetTop(), track)
	title := draw.Pad(ansi.Truncate(" "+m.title(fleetPane, fleetHeader(nameWidth)), a.width-1, ""), a.width-1) + track
	return append([]string{title}, content...)
}
