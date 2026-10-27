package popup

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
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
}

type Result struct {
	Popup  *Station
	Send   []string
	Prompt *command.Target
}

func OpenStation(id string, connector int) (Station, string) {
	return Station{id: id, connector: connector}, command.Describe(id)
}

func (p Station) ID() string {
	return p.id
}

func (p Station) Key(msg tea.KeyPressMsg, s fleet.Station) Result {
	if p.tab == configTab {
		return p.configKey(msg)
	}
	key := msg.String()
	last := max(len(s.Connectors)-1, 0)
	current := min(p.connector, last)
	switch key {
	case "esc", "q":
		return Result{}
	case "tab", "shift+tab":
		p.tab = configTab
	case "j", "down":
		p.connector = min(current+1, last)
	case "k", "up":
		p.connector = max(current-1, 0)
	case "a":
		if len(s.Connectors) > 0 {
			return Result{Popup: &p, Prompt: &command.Target{Station: s.ID, Connector: s.Connectors[current].ID}}
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
