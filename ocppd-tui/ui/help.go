package ui

import (
	"strings"

	"charm.land/lipgloss/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

const helpWidth = 58

func helpLines(p pane) []string {
	var lines []string
	group := ""
	for _, b := range bindingsFor(p) {
		if b.group != group {
			group = b.group
			lines = append(lines, " "+draw.Dim.Render(group))
		}
		lines = append(lines, "   "+draw.Accent.Render(draw.Pad(b.keys, 10))+b.help)
	}
	return lines
}

func (m Model) helpKey(key string) Model {
	switch key {
	case "j", "down":
		m.helpTop = min(m.helpTop+1, max(len(helpLines(m.focus))-m.modalRows(), 0))
	case "k", "up":
		m.helpTop = max(m.helpTop-1, 0)
	case "esc", "?", "q":
		m.help, m.helpTop = false, 0
	}
	return m
}

func keyBar(width int, p pane) string {
	var items []string
	for _, b := range bindingsFor(p) {
		if b.bar != "" {
			items = append(items, draw.Accent.Render(strings.ReplaceAll(b.keys, " / ", "/"))+" "+b.bar)
		}
	}
	help := draw.Accent.Render("?") + " help"
	bar := " "
	for _, item := range items {
		if lipgloss.Width(bar+item+"  "+help) > width {
			break
		}
		bar += item + "  "
	}
	return bar + help
}
