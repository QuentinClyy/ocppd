package popup

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/motion"
)

const (
	detailsTab = iota
	configTab
)

type Station struct {
	id        string
	connector int
	described description
	tab       int
	config    configView
	pending   string
}

type Result struct {
	Popup  *Station
	Send   []string
	Prompt *command.Target
	Plug   *command.Target
}

func OpenStation(id string, connector int) (Station, string) {
	return Station{id: id, connector: connector}, command.Describe(id)
}

func (p Station) ID() string {
	return p.id
}

func (p Station) Typing() bool {
	return p.tab == configTab && (p.config.draft != nil || p.config.filtering)
}

func (p Station) Key(msg tea.KeyPressMsg, s fleet.Station) Result {
	if p.tab == configTab {
		return p.configKey(msg, s.Link == "up")
	}
	pending, count, key := motion.Feed(p.pending, msg.String(), "g")
	p.pending = pending
	last := max(len(s.Connectors)-1, 0)
	current := min(p.connector, last)
	if index, ok := motion.Move(key, count, current, last); ok {
		p.connector = index
		return Result{Popup: &p}
	}
	switch key {
	case "esc", "q":
		return Result{}
	case "tab", "shift+tab", "gt", "gT":
		p.tab = configTab
	case "a":
		if len(s.Connectors) > 0 {
			return Result{Popup: &p, Prompt: &command.Target{Station: s.ID, Connector: s.Connectors[current].ID}}
		}
	case "o":
		return Result{Popup: &p, Send: []string{command.Link(s.ID, s.Held)}}
	case "P":
		if len(s.Connectors) > 0 {
			return Result{Popup: &p, Plug: &command.Target{Station: s.ID, Connector: s.Connectors[current].ID}}
		}
	case "p", "s", "u", "f", "c", "x":
		at := command.Target{Station: s.ID}
		if len(s.Connectors) > 0 {
			at.Connector = s.Connectors[current].ID
		}
		if line, ok := command.Driver(key, at); ok {
			return Result{Popup: &p, Send: []string{line}}
		}
	}
	return Result{Popup: &p}
}
