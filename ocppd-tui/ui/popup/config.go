package popup

import (
	"strings"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/motion"
)

type configView struct {
	row       int
	filter    draw.Editor
	filtering bool
	draft     *draw.Editor
	err       string
}

const csmsKey = "csms"

func (p Station) shown() []setting {
	var shown []setting
	settings := p.described.Configuration
	if p.described.Station != "" {
		csms := strings.TrimSuffix(p.described.Csms, "/"+p.id)
		settings = append([]setting{{Key: csmsKey, Readonly: true, Value: &csms}}, settings...)
	}
	for _, s := range settings {
		if strings.Contains(strings.ToLower(s.Key), strings.ToLower(p.config.filter.String())) {
			shown = append(shown, s)
		}
	}
	return shown
}

func (s setting) isEditable(online bool) bool {
	return !s.Readonly || !online
}

func (s setting) isBoolean() bool {
	return s.Value != nil && (*s.Value == "true" || *s.Value == "false")
}

func (p Station) configKey(msg tea.KeyPressMsg, online bool) Result {
	c := &p.config
	key := msg.String()
	switch {
	case c.draft != nil:
		return p.editKey(msg)
	case c.filtering:
		switch key {
		case "enter":
			c.filtering = false
		case "esc":
			c.filter, c.filtering = draw.Editor{}, false
		default:
			c.filter, c.row = c.filter.Edit(msg), 0
		}
		return Result{Popup: &p}
	}
	shown := p.shown()
	c.row = min(c.row, max(len(shown)-1, 0))
	pending, count, key := motion.Feed(p.pending, key, "g")
	p.pending = pending
	if row, ok := motion.Move(key, count, c.row, max(len(shown)-1, 0)); ok {
		c.row = row
		return Result{Popup: &p}
	}
	switch key {
	case "esc", "q":
		return Result{}
	case "tab", "shift+tab", "gt", "gT":
		p.tab = detailsTab
	case "/":
		c.filtering = true
	case "space":
		if c.row < len(shown) && shown[c.row].isBoolean() && shown[c.row].isEditable(online) {
			flipped := map[string]string{"true": "false", "false": "true"}[*shown[c.row].Value]
			return Result{Popup: &p, Send: []string{command.Configure(p.id, shown[c.row].Key, flipped)}}
		}
	case "i", "enter":
		if c.row < len(shown) && shown[c.row].isEditable(online) {
			draft := draw.Editor{}
			if shown[c.row].Value != nil {
				draft = draw.NewEditor(*shown[c.row].Value)
			}
			c.draft, c.err = &draft, ""
		}
	}
	return Result{Popup: &p}
}

func (p Station) Paste(text string) Station {
	c := &p.config
	switch {
	case p.tab != configTab:
	case c.draft != nil:
		draft := c.draft.Paste(text)
		c.draft = &draft
	case c.filtering:
		c.filter, c.row = c.filter.Paste(text), 0
	}
	return p
}

func (p Station) editKey(msg tea.KeyPressMsg) Result {
	c := &p.config
	key := msg.String()
	if key != "enter" && key != "esc" {
		draft := c.draft.Edit(msg)
		c.draft = &draft
		return Result{Popup: &p}
	}
	draft := c.draft.String()
	c.draft = nil
	shown := p.shown()
	if key == "esc" || c.row >= len(shown) {
		return Result{Popup: &p}
	}
	if shown[c.row].Key == csmsKey {
		return Result{Popup: &p, Send: []string{command.SetCsms(p.id, draft)}}
	}
	return Result{Popup: &p, Send: []string{command.Configure(p.id, shown[c.row].Key, draft)}}
}
