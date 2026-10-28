package draw

import (
	"strings"

	"charm.land/lipgloss/v2"
)

type Binding struct {
	Group string
	Keys  string
	Help  string
	Bar   string
}

func HelpLines(bindings []Binding) []string {
	var lines []string
	group := ""
	for _, b := range bindings {
		if b.Group != group {
			group = b.Group
			lines = append(lines, " "+Dim.Render(group))
		}
		lines = append(lines, "   "+Accent.Render(Pad(b.Keys, 10))+b.Help)
	}
	return lines
}

func KeyBar(width int, bindings []Binding, showcmd string) string {
	room := width
	if showcmd != "" {
		room -= lipgloss.Width(showcmd) + 2
	}
	help := Accent.Render("?") + " help"
	bar := " "
	for _, b := range bindings {
		if b.Bar == "" {
			continue
		}
		item := Accent.Render(strings.ReplaceAll(b.Keys, " / ", "/")) + " " + b.Bar
		if lipgloss.Width(bar+item+"  "+help) > room {
			break
		}
		bar += item + "  "
	}
	return WithShowcmd(bar+help, width, showcmd)
}

func WithShowcmd(bar string, width int, showcmd string) string {
	if showcmd == "" {
		return bar
	}
	return Pad(bar, width-lipgloss.Width(showcmd)-1) + Dim.Render(showcmd)
}
