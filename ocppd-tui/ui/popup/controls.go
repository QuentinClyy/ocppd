package popup

import (
	"strings"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

func statusLine(badge, err string) string {
	if err == "" {
		return badge
	}
	return badge + " " + draw.Warn.Render("! "+err)
}

func segmented(c choice, focused bool) string {
	parts := make([]string, len(c.options))
	for index, option := range c.options {
		style := draw.Dim
		switch {
		case index == c.index && focused:
			style = draw.Chosen
		case index == c.index:
			style = draw.Selected
		}
		parts[index] = style.Render(" " + option + " ")
	}
	return strings.Join(parts, draw.Dim.Render("│"))
}

func section(title string) string {
	return "  " + draw.Heading.Render(title)
}

func keyHints(keys [][2]string) string {
	var items []string
	for _, key := range keys {
		items = append(items, draw.Accent.Render(key[0])+" "+key[1])
	}
	return " " + strings.Join(items, "  ")
}
