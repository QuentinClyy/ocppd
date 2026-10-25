package ui

import (
	"strings"

	"github.com/charmbracelet/x/ansi"
)

func box(title string, lines []string, width int) []string {
	top := "┌─ " + bold.Render(title) + " " + strings.Repeat("─", max(width-5-ansi.StringWidth(title), 0)) + "┐"
	framed := []string{top}
	for _, line := range lines {
		framed = append(framed, "│ "+pad(ansi.Truncate(line, width-4, ""), width-3)+"│")
	}
	return append(framed, "└"+strings.Repeat("─", width-2)+"┘")
}

func centered(framed []string, width, height int) []string {
	indent := strings.Repeat(" ", max((width-ansi.StringWidth(framed[0]))/2, 1))
	lines := make([]string, height)
	for index := range lines {
		if index < len(framed) {
			lines[index] = indent + framed[index]
		}
	}
	return lines
}
