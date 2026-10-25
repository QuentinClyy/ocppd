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
	keyBar     = " " + "j/k move  h/l collapse/expand  gg/G first/last  tab log filter  q quit"
)

func render(m Model) string {
	visible := rows(m.fleet, m.collapsed)
	cursor := min(m.cursor, max(len(visible)-1, 0))
	rule := " " + dim.Render(strings.Repeat("─", max(m.width-2, 0)))
	count := fmt.Sprintf("%d stations", len(m.fleet.Stations))
	header := " " + bold.Render("ocppd") + "  " + dim.Render(m.csms)
	header = pad(header, max(m.width-len(count)-1, 0)) + dim.Render(count)
	body := m.height - 4
	var lines []string
	if m.width >= wideWidth {
		lines = m.wide(visible, cursor, body)
	} else {
		lines = m.narrow(visible, cursor, body-3, rule)
	}
	screen := append([]string{header, rule}, lines...)
	screen = append(screen, rule, dim.Render(keyBar))
	for index, line := range screen {
		screen[index] = ansi.Truncate(line, m.width, "")
	}
	return strings.Join(screen, "\n")
}

func (m Model) narrow(visible []row, cursor, available int, rule string) []string {
	fleetRows := min(len(visible), max(available*2/3, 1))
	shown := window(fleetLines(m.fleet, visible, m.collapsed, cursor, narrowName), cursor, fleetRows)
	lines := []string{" " + fleetHeader(narrowName)}
	for _, line := range shown {
		lines = append(lines, " "+line)
	}
	lines = append(lines, rule, m.logTitle())
	logRows := available - len(shown)
	return append(lines, fill(tailLines(m.logLines(), logRows), logRows)...)
}

func (m Model) wide(visible []row, cursor, body int) []string {
	leftWidth := fleetWidth(wideName)
	left := append([]string{fleetHeader(wideName)}, window(fleetLines(m.fleet, visible, m.collapsed, cursor, wideName), cursor, body-1)...)
	right := append([]string{strings.TrimPrefix(m.logTitle(), " ")}, tailLines(m.logLines(), body-1)...)
	lines := make([]string, body)
	for index := range lines {
		var l, r string
		if index < len(left) {
			l = left[index]
		}
		if index < len(right) {
			r = strings.TrimPrefix(right[index], " ")
		}
		lines[index] = " " + pad(l, leftWidth) + dim.Render("│") + " " + r
	}
	return lines
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

func window(lines []string, cursor, height int) []string {
	start := max(min(cursor-height+1, len(lines)-height), 0)
	return lines[start:min(start+height, len(lines))]
}

func tailLines(lines []string, height int) []string {
	shown := lines[max(len(lines)-max(height, 0), 0):]
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
