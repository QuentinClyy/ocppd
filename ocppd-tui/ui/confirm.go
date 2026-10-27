package ui

import (
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

const shutdown = `{"command":"shutdown"}`

type confirmation struct {
	question string
	command  string
	again    string
	station  string
}

func removal(station string) *confirmation {
	return &confirmation{question: "remove " + station + "?", command: command.Remove(station), again: "d", station: station}
}

func stopping() *confirmation {
	return &confirmation{question: "stop the daemon and every station?", command: shutdown}
}

func (m Model) confirmKey(key string) Model {
	c := m.confirm
	m.confirm = nil
	if key != "y" && (c.again == "" || key != c.again) {
		return m
	}
	if c.station != "" {
		m.fleet.Remove(c.station)
		delete(m.expanded, c.station)
	}
	m.stopping = c.command == shutdown
	return m.sendLine(c.command)
}

func (c *confirmation) bar() string {
	return " " + draw.Warn.Render(c.question) + "  " + draw.Accent.Render("y") + " yes  " + draw.Accent.Render("n") + " no"
}
