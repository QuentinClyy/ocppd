package popup

import (
	"strings"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

type configView struct {
	row       int
	filter    draw.Editor
	filtering bool
	pendingG  bool
	draft     *draw.Editor
	err       string
}

func (p Station) shown() []setting {
	var shown []setting
	for _, s := range p.described.Configuration {
		if strings.Contains(strings.ToLower(s.Key), strings.ToLower(p.config.filter.String())) {
			shown = append(shown, s)
		}
	}
	return shown
}

func (s setting) isBoolean() bool {
	return s.Value != nil && (*s.Value == "true" || *s.Value == "false")
}

func (p Station) configKey(msg tea.KeyPressMsg) Result {
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
	wasG := c.pendingG
	c.pendingG = false
	switch key {
	case "esc", "q":
		return Result{}
	case "tab", "shift+tab":
		p.tab = detailsTab
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
			return Result{Popup: &p, Send: []string{command.Configure(p.id, shown[c.row].Key, flipped)}}
		}
	case "i", "enter":
		if c.row < len(shown) && !shown[c.row].Readonly {
			draft := draw.Editor{}
			if shown[c.row].Value != nil {
				draft = draw.NewEditor(*shown[c.row].Value)
			}
			c.draft, c.err = &draft, ""
		}
	}
	return Result{Popup: &p}
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
	return Result{Popup: &p, Send: []string{command.Configure(p.id, shown[c.row].Key, draft)}}
}
