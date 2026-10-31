package popup

import (
	"errors"
	"fmt"
	"strings"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

const plugWidth = 44

type Plug struct {
	at       command.Target
	capacity draw.Editor
	soc      draw.Editor
	target   draw.Editor
	packV    draw.Editor
	acKW     draw.Editor
	acPhases draw.Editor
	dcKW     draw.Editor
	fields
}

func NewPlug(at command.Target) *Plug {
	return &Plug{at: at}
}

func (f *Plug) slots() []slot {
	return []slot{
		{label: "capacity kWh", text: &f.capacity, hint: "60", width: 8},
		{label: "soc %", text: &f.soc, hint: "20", width: 8},
		{label: "target %", text: &f.target, hint: "100", width: 8},
		{label: "pack V", text: &f.packV, hint: "400", width: 8},
		{label: "ac kW", text: &f.acKW, hint: "11", width: 8},
		{label: "ac phases", text: &f.acPhases, hint: "3", width: 8},
		{label: "dc kW", text: &f.dcKW, hint: "150", width: 8},
	}
}

func (f *Plug) Key(msg tea.KeyPressMsg) (string, bool) {
	return f.submit(msg, f.slots(), func(string) {}, f.line)
}

func (f *Plug) Paste(text string) {
	f.paste(f.slots(), text)
}

func (f *Plug) Pastes(key string) bool {
	return f.pastes(key)
}

func (f *Plug) Typing() bool {
	return !f.normal
}

func (f *Plug) line() (string, error) {
	ev := map[string]any{}
	numbers := []struct {
		key, label, text string
		scale            float64
		isWhole          bool
	}{
		{"capacity_kwh", "capacity kWh", f.capacity.String(), 1, false},
		{"soc", "soc %", f.soc.String(), 100, false},
		{"target_soc", "target %", f.target.String(), 100, false},
		{"pack_v", "pack V", f.packV.String(), 1, false},
		{"ac_kw", "ac kW", f.acKW.String(), 1, false},
		{"ac_phases", "ac phases", f.acPhases.String(), 1, true},
		{"dc_kw", "dc kW", f.dcKW.String(), 1, false},
	}
	for _, n := range numbers {
		if strings.TrimSpace(n.text) == "" {
			continue
		}
		value, err := parseNumber(n.text, n.isWhole)
		if err != nil {
			return "", errors.New(n.label + " must be a number")
		}
		if number, ok := value.(float64); ok {
			value = number / n.scale
		}
		ev[n.key] = value
	}
	fields := map[string]any{"command": "plug_in", "station": f.at.Station, "connector": f.at.Connector}
	if len(ev) > 0 {
		fields["ev"] = ev
	}
	return command.Encode(fields), nil
}

func (f *Plug) Frame(rows int) draw.Frame {
	slots := f.slots()
	f.focus = min(f.focus, len(slots)-1)
	lines := []string{"", section("BATTERY")}
	focused := 0
	for index, s := range slots {
		if s.label == "ac kW" {
			lines = append(lines, "", section("ONBOARD AND DC LIMITS"))
		}
		if index == f.focus {
			focused = len(lines)
		}
		lines = append(lines, f.marker(index, 1)+f.label(s.label, index, 14)+f.value(s, index))
	}
	lines = append(lines, "")
	title := draw.Bold.Render(fmt.Sprintf("plug in · %s #%d", f.at.Station, f.at.Connector))
	return draw.Frame{Title: title, Footer: f.footer(), Lines: lines, Top: focused - rows + 1, Width: plugWidth}
}

func (f *Plug) Bindings() []draw.Binding {
	return plugBindings
}

func (f *Plug) Bar(width int) string {
	if f.normal {
		return draw.KeyBar(width, plugBindings, f.pending)
	}
	return keyHints([][2]string{{"tab", "next"}, {"⇧tab", "back"}, {"enter", "plug in"}, {"esc", "normal mode"}})
}
