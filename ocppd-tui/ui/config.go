package ui

import (
	"strings"

	tea "charm.land/bubbletea/v2"
)

type configView struct {
	row       int
	filter    string
	filtering bool
	pendingG  bool
	draft     *string
	err       string
}

func (d details) shown() []setting {
	var shown []setting
	for _, s := range d.described.Configuration {
		if strings.Contains(strings.ToLower(s.Key), strings.ToLower(d.config.filter)) {
			shown = append(shown, s)
		}
	}
	return shown
}

func (s setting) isBoolean() bool {
	return s.Value != nil && (*s.Value == "true" || *s.Value == "false")
}

func (m Model) configKey(msg tea.KeyPressMsg) Model {
	next := *m.details
	c := &next.config
	key := msg.String()
	switch {
	case c.draft != nil:
		return m.editKey(next, msg)
	case c.filtering:
		c.filter, c.filtering = edited(c.filter, msg)
		c.row = 0
		if key == "esc" {
			c.filter = ""
		}
		m.details = &next
		return m
	}
	shown := next.shown()
	c.row = min(c.row, max(len(shown)-1, 0))
	wasG := c.pendingG
	c.pendingG = false
	switch key {
	case "esc", "q":
		m.details = nil
		return m
	case "tab", "shift+tab":
		next.tab = detailsTab
	case "j", "down":
		c.row = min(c.row+1, max(len(shown)-1, 0))
	case "k", "up":
		c.row = max(c.row-1, 0)
	case "g":
		if wasG {
			c.row = 0
		}
		c.pendingG = !wasG
	case "G":
		c.row = max(len(shown)-1, 0)
	case "/":
		c.filtering = true
	case "space":
		if c.row < len(shown) && shown[c.row].isBoolean() && !shown[c.row].Readonly {
			flipped := map[string]string{"true": "false", "false": "true"}[*shown[c.row].Value]
			m.details = &next
			return m.configure(shown[c.row].Key, flipped)
		}
	case "i", "enter":
		if c.row < len(shown) && !shown[c.row].Readonly {
			draft := ""
			if shown[c.row].Value != nil {
				draft = *shown[c.row].Value
			}
			c.draft, c.err = &draft, ""
		}
	}
	m.details = &next
	return m
}

func (m Model) editKey(next details, msg tea.KeyPressMsg) Model {
	c := &next.config
	draft, typing := edited(*c.draft, msg)
	if typing {
		c.draft = &draft
		m.details = &next
		return m
	}
	c.draft = nil
	m.details = &next
	shown := next.shown()
	if msg.String() == "esc" || c.row >= len(shown) {
		return m
	}
	return m.configure(shown[c.row].Key, draft)
}

func edited(text string, msg tea.KeyPressMsg) (string, bool) {
	switch msg.String() {
	case "enter", "esc":
		return text, false
	case "backspace":
		runes := []rune(text)
		return string(runes[:max(len(runes)-1, 0)]), true
	}
	return text + msg.Text, true
}

func (m Model) configure(key, value string) Model {
	return m.sendLine(encode(map[string]any{"command": "configure", "station": m.details.station, "key": key, "value": value}))
}
