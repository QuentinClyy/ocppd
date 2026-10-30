package popup

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

type choice struct {
	options []string
	index   int
}

type credential struct {
	id       draw.Editor
	password draw.Editor
}

type Spawn struct {
	listed     choice
	stations   []credential
	prefix     draw.Editor
	count      draw.Editor
	password   draw.Editor
	ocpp       choice
	current    choice
	vendor     draw.Editor
	model      draw.Editor
	phases     draw.Editor
	maxCurrent draw.Editor
	connectors draw.Editor
	voltage    draw.Editor
	maxOutput  draw.Editor
	csms       draw.Editor
	connect    choice
	fields
}

func NewSpawn() *Spawn {
	return &Spawn{
		listed:   choice{options: []string{"list", "prefix + count"}},
		stations: []credential{{}},
		ocpp:     choice{options: []string{"1.6", "2.0.1"}},
		current:  choice{options: []string{"ac", "dc"}},
		connect:  choice{options: []string{"yes", "no"}},
	}
}

func (f *Spawn) Key(msg tea.KeyPressMsg) (string, bool) {
	return f.submit(msg, f.slots(), f.addStation, f.line)
}

func (f *Spawn) Paste(text string) {
	f.paste(f.slots(), text)
}

func (f *Spawn) Pastes(key string) bool {
	return f.pastes(key)
}

func (f *Spawn) Typing() bool {
	return !f.normal
}

func (f *Spawn) addStation(string) {
	if f.isListed() {
		f.stations = append(f.stations, credential{})
		f.focus = 2*len(f.stations) - 1
	}
}

func (f *Spawn) isDC() bool {
	return f.current.index == 1
}

func (f *Spawn) isListed() bool {
	return f.listed.index == 0
}

func (f *Spawn) slots() []slot {
	slots := []slot{{label: "entry", choice: &f.listed}}
	if f.isListed() {
		for index := range f.stations {
			slots = append(slots,
				slot{label: "id", text: &f.stations[index].id, width: 18},
				slot{label: "password", text: &f.stations[index].password, width: 18})
		}
	} else {
		slots = append(slots,
			slot{label: "prefix", text: &f.prefix, hint: "CP", width: 24},
			slot{label: "count", text: &f.count, hint: "1", width: 6},
			slot{label: "password", text: &f.password, width: 24})
	}
	slots = append(slots,
		slot{label: "ocpp", choice: &f.ocpp},
		slot{label: "current", choice: &f.current},
		slot{label: "vendor", text: &f.vendor, hint: "ocppd", width: 24},
		slot{label: "model", text: &f.model, hint: "sim", width: 24},
		slot{label: "voltage V", text: &f.voltage, hint: "230", width: 6})
	if !f.isDC() {
		slots = append(slots, slot{label: "phases", text: &f.phases, hint: "1", width: 6})
	}
	return append(slots,
		slot{label: "max A", text: &f.maxCurrent, hint: "32", width: 6},
		slot{label: "connectors", text: &f.connectors, hint: "1", width: 6},
		slot{label: "dc out A", text: &f.maxOutput, hint: "500", width: 6},
		slot{label: "csms", text: &f.csms, hint: "default (--csms)", width: 46},
		slot{label: "connect", choice: &f.connect},
	)
}
