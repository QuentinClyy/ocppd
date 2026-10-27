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
	vendor     draw.Editor
	model      draw.Editor
	phases     draw.Editor
	maxCurrent draw.Editor
	connectors draw.Editor
	csms       draw.Editor
	focus      int
	normal     bool
	pendingG   bool
	err        string
}

type slot struct {
	label  string
	text   *draw.Editor
	choice *choice
	hint   string
	width  int
}

func NewSpawn() *Spawn {
	return &Spawn{
		listed:   choice{options: []string{"list", "prefix + count"}},
		stations: []credential{{}},
		ocpp:     choice{options: []string{"1.6", "2.0.1"}},
	}
}

func (f *Spawn) Key(msg tea.KeyPressMsg) (string, bool) {
	key := msg.String()
	switch {
	case key == "esc" && !f.normal:
		f.normal = true
		return "", false
	case key == "esc" || key == "q" && f.normal:
		return "", true
	}
	if !f.edit(msg) {
		return "", false
	}
	line, err := f.line()
	if err != nil {
		f.err = err.Error()
		return "", false
	}
	return line, true
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
	return append(slots,
		slot{label: "ocpp", choice: &f.ocpp},
		slot{label: "vendor", text: &f.vendor, hint: "ocppd", width: 24},
		slot{label: "model", text: &f.model, hint: "sim", width: 24},
		slot{label: "phases", text: &f.phases, hint: "3", width: 6},
		slot{label: "max A", text: &f.maxCurrent, hint: "32", width: 6},
		slot{label: "connectors", text: &f.connectors, hint: "1", width: 6},
		slot{label: "csms", text: &f.csms, hint: "default (--csms)", width: 46},
	)
}

func (f *Spawn) edit(key tea.KeyPressMsg) bool {
	slots := f.slots()
	f.focus = min(f.focus, len(slots)-1)
	if f.normal {
		return f.normalKey(key.String(), slots)
	}
	current := slots[f.focus]
	switch key.String() {
	case "enter":
		return true
	case "tab":
		f.focus = (f.focus + 1) % len(slots)
	case "shift+tab":
		f.focus = (f.focus + len(slots) - 1) % len(slots)
	case "ctrl+n":
		if f.isListed() {
			f.stations = append(f.stations, credential{})
			f.focus = 2*len(f.stations) - 1
		}
	case "space", "left", "right":
		if current.choice != nil {
			current.choice.index = (current.choice.index + 1) % len(current.choice.options)
		} else {
			*current.text = current.text.Edit(key)
		}
	default:
		if current.text != nil {
			*current.text = current.text.Edit(key)
		}
	}
	f.err = ""
	return false
}

func (f *Spawn) normalKey(key string, slots []slot) bool {
	wasG := f.pendingG
	f.pendingG = false
	current := slots[f.focus]
	switch key {
	case "enter":
		return true
	case "j", "down", "tab":
		f.focus = min(f.focus+1, len(slots)-1)
	case "k", "up", "shift+tab":
		f.focus = max(f.focus-1, 0)
	case "g":
		if wasG {
			f.focus = 0
		}
		f.pendingG = !wasG
	case "G":
		f.focus = len(slots) - 1
	case "h", "l", "left", "right":
		if current.choice != nil {
			current.choice.index = (current.choice.index + 1) % len(current.choice.options)
		}
	case "i", "a":
		f.normal = false
	case "ctrl+n":
		if f.isListed() {
			f.stations = append(f.stations, credential{})
			f.focus = 2*len(f.stations) - 1
		}
	}
	f.err = ""
	return false
}
