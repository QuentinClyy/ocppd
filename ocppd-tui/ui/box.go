package ui

import (
	"strings"

	"github.com/charmbracelet/x/ansi"
)

const minModalWidth = 16

type popup struct {
	title  string
	footer string
	lines  []string
	top    int
	width  int
}

func box(title, footer string, lines []string, width int) []string {
	framed := []string{dim.Render("╭─ ") + bold.Render(title) + dim.Render(" "+strings.Repeat("─", max(width-5-ansi.StringWidth(title), 0))+"╮")}
	for _, line := range lines {
		framed = append(framed, dim.Render("│")+pad(ansi.Truncate(line, width-2, ""), width-2)+dim.Render("│"))
	}
	bottom := dim.Render("╰" + strings.Repeat("─", max(width-2, 0)) + "╯")
	if footer != "" {
		footer = ansi.Truncate(footer, max(width-6, 0), "…")
		bottom = dim.Render("╰─ ") + footer + dim.Render(" "+strings.Repeat("─", max(width-5-ansi.StringWidth(footer), 0))+"╯")
	}
	return append(framed, bottom)
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

func (m Model) modal(p popup) []string {
	rows := m.modalRows()
	top := min(max(p.top, 0), max(len(p.lines)-rows, 0))
	framed := box(p.title, p.footer, window(p.lines, top, rows), max(min(p.width, m.width-2), minModalWidth))
	return centered(framed, m.width, m.height-4)
}

func (m Model) modalRows() int {
	return max(m.height-6, 1)
}
