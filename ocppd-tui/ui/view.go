package ui

import (
	"fmt"
	"strings"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/charmbracelet/x/ansi"
)

const (
	wideWidth   = 150
	narrowName  = 24
	wideName    = 20
	filterWidth = 40
)

func render(m Model) string {
	rule := " " + draw.Dim.Render(strings.Repeat("─", max(m.width-2, 0)))
	right := draw.Dim.Render(m.stationCount())
	if m.outdated() {
		right = draw.Warn.Render("daemon "+m.daemon+" ≠ "+m.expected+" · Q restarts") + "  " + right
	}
	header := " " + draw.Bold.Render("OCPPD") + "  " + draw.Dim.Render(m.csms)
	room := max(m.width-ansi.StringWidth(right)-2, 0)
	header = draw.Pad(ansi.Truncate(header, room, "…"), room+1) + right
	var lines []string
	switch {
	case m.form != nil:
		lines = m.modal(m.form.Frame(m.modalRows()))
	case m.details != nil:
		lines = m.detailsFrame()
	case m.help:
		lines = m.modal(draw.Frame{Title: draw.Bold.Render("keys"), Lines: helpLines(m.focus), Top: m.helpTop, Width: helpWidth})
	default:
		lines = m.body(m.layout(), rule)
	}
	bar := keyBar(m.width, m.focus)
	switch {
	case m.prompt != nil:
		bar = m.prompt.view()
	case m.filtering:
		bar = " " + draw.Accent.Render("/") + " " + m.filters[m.focus].View(filterWidth) + "   " + draw.Accent.Render("enter") + " keep  " + draw.Accent.Render("esc") + " clear"
	case m.form != nil:
		bar = m.form.Bar()
	case m.confirm != nil:
		bar = m.confirm.bar()
	case m.details != nil:
		bar = m.details.Bar()
	case m.help:
		bar = " " + draw.Accent.Render("j/k") + " scroll  " + draw.Accent.Render("esc") + " close"
	}
	screen := append([]string{header, rule}, lines...)
	screen = append(screen, rule, bar)
	for index, line := range screen {
		screen[index] = ansi.Truncate(line, m.width, "")
	}
	return strings.Join(screen, "\n")
}

func (m Model) stationCount() string {
	count := fmt.Sprintf("%d stations", len(m.fleet.Stations))
	if len(m.fleet.Stations) == 1 {
		count = "1 station"
	}
	filter := m.filters[fleetPane].String()
	if filter == "" {
		return count
	}
	shown := 0
	for _, r := range m.visibleRows() {
		if r.connector == stationRow {
			shown++
		}
	}
	return fmt.Sprintf("%d of %s · /%s", shown, count, filter)
}

func (m Model) body(l layout, rule string) []string {
	var panes [][]string
	for _, a := range l.areas {
		panes = append(panes, m.paneLines(a))
	}
	if l.side {
		return besides(panes)
	}
	var lines []string
	for index, pane := range panes {
		if index > 0 {
			lines = append(lines, rule)
		}
		lines = append(lines, pane...)
	}
	return lines
}

func besides(panes [][]string) []string {
	lines := make([]string, len(panes[0]))
	for _, pane := range panes {
		for index, line := range pane {
			lines[index] += line
		}
	}
	return lines
}

func (m Model) paneLines(a area) []string {
	switch a.pane {
	case fleetPane:
		return m.fleetPane(a)
	case logPane:
		return m.logPane(a)
	}
	return nil
}

func (m Model) title(p pane, text string) string {
	if m.focus == p {
		return draw.Bold.Render(text)
	}
	return draw.Dim.Render(text)
}
