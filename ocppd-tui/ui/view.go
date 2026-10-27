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
	rule := " " + dim.Render(strings.Repeat("─", max(m.width-2, 0)))
	count := fmt.Sprintf("%d stations", len(m.fleet.Stations))
	if len(m.fleet.Stations) == 1 {
		count = "1 station"
	}
	right := dim.Render(count)
	if m.outdated() {
		right = warn.Render("daemon "+m.daemon+" ≠ "+m.expected+" · Q restarts") + "  " + right
	}
	header := " " + bold.Render("ocppd") + "  " + dim.Render(m.csms)
	room := max(m.width-ansi.StringWidth(right)-2, 0)
	header = pad(ansi.Truncate(header, room, "…"), room+1) + right
	var lines []string
	switch {
	case m.form != nil:
		fields, focused := m.form.lines()
		lines = m.modal(popup{title: "spawn", footer: m.form.footer(), lines: fields, top: focused - m.modalRows() + 1, width: formWidth})
	case m.details != nil:
		lines = m.modal(popup{title: m.details.station, lines: m.detailsLines(), width: detailsWidth})
	case m.help:
		lines = m.modal(popup{title: "keys", lines: helpLines(m.focus), top: m.helpTop, width: helpWidth})
	default:
		lines = m.body(m.layout(), rule)
	}
	bar := keyBar(m.width, m.focus)
	switch {
	case m.prompt != nil:
		bar = m.prompt.view()
	case m.form != nil:
		bar = m.form.bar()
	case m.confirm != nil:
		bar = m.confirm.bar()
	case m.details != nil:
		bar = detailsBar()
	case m.help:
		bar = " " + accent.Render("j/k") + " scroll  " + accent.Render("esc") + " close"
	}
	screen := append([]string{header, rule}, lines...)
	screen = append(screen, rule, bar)
	for index, line := range screen {
		screen[index] = ansi.Truncate(line, m.width, "")
	}
	return strings.Join(screen, "\n")
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
		return bold.Render(text)
	}
	return dim.Render(text)
}
