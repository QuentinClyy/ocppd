package ui

import (
	"strings"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/charmbracelet/x/ansi"
)

type point struct{ x, y int }

type selection struct {
	bounds   area
	from, to point
	live     bool
}

func (s selection) extend(x, y int) selection {
	s.to = point{
		x: min(max(x, s.bounds.x), s.bounds.x+s.bounds.width-1),
		y: min(max(y, s.bounds.y), s.bounds.y+s.bounds.height-1),
	}
	return s
}

func (s selection) moved() bool {
	return s.from != s.to
}

func (s selection) span() (start, end point) {
	if s.to.y < s.from.y || (s.to.y == s.from.y && s.to.x < s.from.x) {
		return s.to, s.from
	}
	return s.from, s.to
}

func (s selection) columns(y int) (left, right int) {
	start, end := s.span()
	left, right = s.bounds.x, s.bounds.x+s.bounds.width
	if y == start.y {
		left = start.x
	}
	if y == end.y {
		right = end.x + 1
	}
	return left, right
}

func selectedText(lines []string, s selection) string {
	start, end := s.span()
	var picked []string
	for y := start.y; y <= end.y && y < len(lines); y++ {
		left, right := s.columns(y)
		picked = append(picked, strings.TrimRight(ansi.Strip(ansi.Cut(lines[y], left, right)), " "))
	}
	return strings.TrimRight(strings.Join(picked, "\n"), "\n")
}

func highlight(lines []string, s selection) []string {
	if !s.live {
		return lines
	}
	shown := append([]string(nil), lines...)
	start, end := s.span()
	for y := start.y; y <= end.y && y < len(shown); y++ {
		left, right := s.columns(y)
		line := shown[y]
		shown[y] = ansi.Cut(line, 0, left) +
			draw.Highlight.Render(ansi.Strip(ansi.Cut(line, left, right))) +
			ansi.Cut(line, right, ansi.StringWidth(line))
	}
	return shown
}
