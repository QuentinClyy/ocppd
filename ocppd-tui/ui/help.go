package ui

import (
	"strings"

	"charm.land/lipgloss/v2"
)

const helpWidth = 58

func helpLines() []string {
	lines := []string{"┌─ " + bold.Render("keys") + " " + strings.Repeat("─", helpWidth-9) + "┐"}
	group := ""
	for _, b := range bindings {
		if b.group != group {
			group = b.group
			lines = append(lines, boxed(dim.Render(group)))
		}
		lines = append(lines, boxed("  "+accent.Render(pad(b.keys, 10))+b.help))
	}
	return append(lines, "└"+strings.Repeat("─", helpWidth-2)+"┘")
}

func boxed(text string) string {
	return "│ " + pad(text, helpWidth-3) + "│"
}

func keyBar(width int) string {
	items := []string{accent.Render("hjkl") + " move"}
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
