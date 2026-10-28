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

func keyBar(width int, p pane, showcmd string) string {
	var items []string
	for _, b := range bindingsFor(p) {
		if b.bar != "" {
			items = append(items, draw.Accent.Render(strings.ReplaceAll(b.keys, " / ", "/"))+" "+b.bar)
		}
	}
	room := width
	if showcmd != "" {
		room -= lipgloss.Width(showcmd) + 2
	}
	help := draw.Accent.Render("?") + " help"
	bar := " "
	for _, item := range items {
		if lipgloss.Width(bar+item+"  "+help) > room {
			break
		}
		bar += item + "  "
	}
	return withShowcmd(bar+help, width, showcmd)
}

func withShowcmd(bar string, width int, showcmd string) string {
	if showcmd == "" {
		return bar
	}
	return draw.Pad(bar, width-lipgloss.Width(showcmd)-1) + draw.Dim.Render(showcmd)
}

func visualBar(width int, showcmd string) string {
	bar := " " + draw.Bold.Render("-- VISUAL LINE --") + "  " + draw.Accent.Render("y") + " yank  " + draw.Accent.Render("esc") + " cancel"
	return withShowcmd(bar, width, showcmd)
}
