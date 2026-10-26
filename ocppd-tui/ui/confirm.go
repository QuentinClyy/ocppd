package ui

import "encoding/json"

const shutdown = `{"command":"shutdown"}`

type confirmation struct {
	question string
	command  string
	again    string
	station  string
}

func removal(station string) *confirmation {
	command := encode(map[string]any{"command": "remove", "station": station})
	return &confirmation{question: "remove " + station + "?", command: command, again: "d", station: station}
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
	return " " + warn.Render(c.question) + "  " + accent.Render("y") + " yes  " + accent.Render("n") + " no"
}

func daemonURL(line string) (string, bool) {
	var greeting struct{ Kind, Csms string }
	if json.Unmarshal([]byte(line), &greeting) != nil || greeting.Kind != "daemon" {
		return "", false
	}
	return greeting.Csms, true
}
