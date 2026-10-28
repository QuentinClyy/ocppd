package ui

import (
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/motion"
)

const helpWidth = 58

func (m Model) helpBindings() []draw.Binding {
	switch {
	case m.form != nil:
		return m.form.Bindings()
	case m.details != nil:
		return m.details.Bindings()
	}
	return bindingsFor(m.focus)
}

func (m Model) helpKey(key string) Model {
	pending, count, key := motion.Feed(m.pending, key, "g")
	m.pending = pending
	last := max(len(draw.HelpLines(m.helpBindings()))-m.modalRows(), 0)
	if top, ok := motion.Move(key, count, m.helpTop, last); ok {
		m.helpTop = top
		return m
	}
	switch key {
	case "esc", "?", "q":
		m.help, m.helpTop = false, 0
	}
	return m
}

func (m Model) opensPopupHelp(key string) bool {
	if key != "?" {
		return false
	}
	return m.form != nil && !m.form.Typing() || m.details != nil && !m.details.Typing()
}

func visualBar(width int, showcmd string) string {
	bar := " " + draw.Bold.Render("-- VISUAL LINE --") + "  " + draw.Accent.Render("y") + " yank  " + draw.Accent.Render("esc") + " cancel"
	return draw.WithShowcmd(bar, width, showcmd)
}
