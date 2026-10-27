package draw

import (
	"strings"

	"charm.land/lipgloss/v2"
)

func Pad(text string, width int) string {
	return text + strings.Repeat(" ", max(width-lipgloss.Width(text), 0))
}

func Dash(text string) string {
	if text == "" {
		return "—"
	}
	return text
}

func Tail(text string, width int) string {
	runes := []rune(text)
	return string(runes[max(len(runes)-width, 0):])
}
