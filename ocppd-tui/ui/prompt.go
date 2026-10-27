package ui

import (
	"fmt"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

const promptWidth = 40

type prompt struct {
	at    command.Target
	value draw.Editor
}

func (p prompt) edit(key tea.KeyPressMsg) (prompt, bool) {
	if key.String() == "enter" {
		return p, true
	}
	p.value = p.value.Edit(key)
	return p, false
}

func (p prompt) view() string {
	label := fmt.Sprintf(" for %s · connector %d › ", p.at.Station, p.at.Connector)
	return " " + draw.Accent.Render("id tag") + draw.Dim.Render(label) + p.value.View(promptWidth) +
		"   " + draw.Accent.Render("enter") + " send  " + draw.Accent.Render("esc") + " cancel"
}
