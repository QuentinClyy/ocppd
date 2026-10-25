package ui

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/x/ansi"
)

const (
	wideWidth  = 150
	narrowName = 24
	wideName   = 20
)

func render(m Model) string {
	visible := rows(m.fleet, m.expanded)
	cursor := min(m.cursor, max(len(visible)-1, 0))
	rule := " " + dim.Render(strings.Repeat("─", max(m.width-2, 0)))
	count := fmt.Sprintf("%d stations", len(m.fleet.Stations))
	if len(m.fleet.Stations) == 1 {
		count = "1 station"
	}
	header := " " + bold.Render("ocppd") + "  " + dim.Render(m.csms)
	header = pad(header, max(m.width-len(count)-1, 0)) + dim.Render(count)
	l := m.layout()
	var lines []string
	switch {
	case m.form != nil:
		lines = centered(m.form.lines(), m.width, m.height-4)
	case m.help:
		lines = centered(helpLines(), m.width, m.height-4)
	case l.wide:
		lines = m.wide(visible, cursor, l)
	default:
		lines = m.narrow(visible, cursor, l, rule)
	}
	bar := keyBar(m.width)
	switch {
	case m.prompt != nil:
		bar = m.prompt.view()
	case m.form != nil:
		bar = formBar()
	case m.help:
		bar = " " + accent.Render("esc") + " close"
	}
	screen := append([]string{header, rule}, lines...)
	screen = append(screen, rule, bar)
	for index, line := range screen {
		screen[index] = ansi.Truncate(line, m.width, "")
	}
	return strings.Join(screen, "\n")
}

func (m Model) narrow(visible []row, cursor int, l layout, rule string) []string {
	shown := window(fleetLines(m.fleet, visible, m.expanded, cursor, narrowName), m.fleetTop(), l.fleetRows)
	fleet := make([]string, len(shown))
	for index, line := range shown {
		fleet[index] = " " + line
	}
	lines := append([]string{" " + fleetHeader(narrowName)}, scrollbar(fleet, m.width, len(visible), m.fleetTop(), " ")...)
	lines = append(lines, rule, m.logTitle())
	return append(lines, m.logPane(l, m.width)...)
}

func (m Model) wide(visible []row, cursor int, l layout) []string {
	leftWidth := fleetWidth(wideName) + 2
	shown := window(fleetLines(m.fleet, visible, m.expanded, cursor, wideName), m.fleetTop(), l.fleetRows)
	left := make([]string, l.fleetRows)
	for index, line := range shown {
		left[index] = " " + line
	}
	left = scrollbar(left, leftWidth, len(visible), m.fleetTop(), dim.Render("│"))
	right := m.logPane(l, m.width-leftWidth)
	lines := []string{pad(" "+fleetHeader(wideName), leftWidth-1) + dim.Render("│") + m.logTitle()}
	for index := range left {
		lines = append(lines, left[index]+right[index])
	}
	return lines
}

func (m Model) logPane(l layout, width int) []string {
	logs := m.logLines()
	offset := m.clampLog(m.logOffset)
	top := max(len(logs)-l.logRows-offset, 0)
	return scrollbar(fill(tailLines(logs, l.logRows, offset), l.logRows), width, len(logs), top, " ")
}

func (m Model) logTitle() string {
	if station := m.logStation(); station != "" {
		return " " + bold.Render(station) + dim.Render(" · log")
	}
	return " " + bold.Render("all stations") + dim.Render(" · log")
}

func (m Model) logLines() []string {
	return filtered(m.log, m.logStation())
}

func window(lines []string, top, height int) []string {
	top = min(top, len(lines))
	return lines[top:min(top+height, len(lines))]
}

func tailLines(lines []string, height, offset int) []string {
	end := max(len(lines)-offset, 0)
	shown := lines[max(end-max(height, 0), 0):end]
	out := make([]string, len(shown))
	for index, line := range shown {
		out[index] = " " + line
	}
	return out
}

func fill(lines []string, height int) []string {
	for len(lines) < height {
		lines = append(lines, "")
	}
	return lines
}
