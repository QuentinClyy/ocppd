package ui

import (
	"fmt"

	tea "charm.land/bubbletea/v2"
)

type prompt struct {
	at    target
	value []rune
}

func (p prompt) edit(key tea.KeyPressMsg) (prompt, bool) {
	switch key.String() {
	case "backspace":
		if len(p.value) > 0 {
			p.value = p.value[:len(p.value)-1]
		}
	case "enter":
		return p, true
	default:
		p.value = append(p.value, []rune(key.Text)...)
	}
	return p, false
}

func (p prompt) view() string {
	label := fmt.Sprintf(" for %s · connector %d › ", p.at.station, p.at.connector)
	return " " + accent.Render("id tag") + dim.Render(label) + string(p.value) + cursorBlock.Render(" ") +
		"   " + accent.Render("enter") + " send  " + accent.Render("esc") + " cancel"
}
