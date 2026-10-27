package draw

import (
	"strings"

	"github.com/charmbracelet/x/ansi"
)

type Frame struct {
	Title  string
	Footer string
	Lines  []string
	Top    int
	Width  int
}

func Box(title, footer string, lines []string, width int) []string {
	framed := []string{Dim.Render("╭─ ") + title + Dim.Render(" "+strings.Repeat("─", max(width-5-ansi.StringWidth(title), 0))+"╮")}
	for _, line := range lines {
		framed = append(framed, Dim.Render("│")+Pad(ansi.Truncate(line, width-2, ""), width-2)+Dim.Render("│"))
	}
	bottom := Dim.Render("╰" + strings.Repeat("─", max(width-2, 0)) + "╯")
	if footer != "" {
		footer = ansi.Truncate(footer, max(width-6, 0), "…")
		bottom = Dim.Render("╰─ ") + footer + Dim.Render(" "+strings.Repeat("─", max(width-5-ansi.StringWidth(footer), 0))+"╯")
	}
	return append(framed, bottom)
}

func Centered(framed []string, width, height int) []string {
	indent := strings.Repeat(" ", max((width-ansi.StringWidth(framed[0]))/2, 1))
	lines := make([]string, height)
	for index := range lines {
		if index < len(framed) {
			lines[index] = indent + framed[index]
		}
	}
	return lines
}
