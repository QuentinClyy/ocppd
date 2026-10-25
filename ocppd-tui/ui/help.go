package ui

import "charm.land/lipgloss/v2"

const helpWidth = 58

func helpLines() []string {
	var lines []string
	group := ""
	for _, b := range bindings {
		if b.group != group {
			group = b.group
			lines = append(lines, dim.Render(group))
		}
		lines = append(lines, "  "+accent.Render(pad(b.keys, 10))+b.help)
	}
	return box("keys", lines, helpWidth)
}

func keyBar(width int) string {
	items := []string{accent.Render("j/k") + " move"}
	for _, b := range bindings {
		if b.bar != "" {
			items = append(items, accent.Render(b.keys)+" "+b.bar)
		}
	}
	help := accent.Render("?") + " help"
	bar := " "
	for _, item := range items {
		if lipgloss.Width(bar+item+"  "+help) > width {
			break
		}
		bar += item + "  "
	}
	return bar + help
}
