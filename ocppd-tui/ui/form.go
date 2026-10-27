package ui

import tea "charm.land/bubbletea/v2"

type choice struct {
	options []string
	index   int
}

type credential struct {
	id       string
	password string
}

type form struct {
	listed     choice
	stations   []credential
	prefix     string
	count      string
	password   string
	ocpp       choice
	vendor     string
	model      string
	phases     string
	maxCurrent string
	connectors string
	csms       string
	focus      int
	normal     bool
	pendingG   bool
	err        string
}

type slot struct {
	label  string
	text   *string
	choice *choice
	secret bool
	hint   string
}

func newForm() *form {
	return &form{
		listed:   choice{options: []string{"list", "prefix + count"}},
		stations: []credential{{}},
		ocpp:     choice{options: []string{"1.6", "2.0.1"}},
	}
}

func (f *form) isListed() bool {
	return f.listed.index == 0
}

func (f *form) slots() []slot {
	slots := []slot{{label: "stations", choice: &f.listed}}
	if f.isListed() {
		for index := range f.stations {
			slots = append(slots,
				slot{label: "id", text: &f.stations[index].id},
				slot{label: "password", text: &f.stations[index].password, secret: true})
		}
	} else {
		slots = append(slots,
			slot{label: "prefix", text: &f.prefix, hint: "CP"},
			slot{label: "count", text: &f.count, hint: "1"},
			slot{label: "password", text: &f.password, secret: true})
	}
	return append(slots,
		slot{label: "ocpp", choice: &f.ocpp},
		slot{label: "vendor", text: &f.vendor, hint: "ocppd"},
		slot{label: "model", text: &f.model, hint: "sim"},
		slot{label: "phases", text: &f.phases, hint: "3"},
		slot{label: "max A", text: &f.maxCurrent, hint: "32"},
		slot{label: "connectors", text: &f.connectors, hint: "1"},
		slot{label: "csms", text: &f.csms, hint: "default (--csms)"},
	)
}

func (f *form) edit(key tea.KeyPressMsg) bool {
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
	case "backspace":
		if current.text != nil {
			runes := []rune(*current.text)
			*current.text = string(runes[:max(len(runes)-1, 0)])
		}
	case "space", "left", "right":
		if current.choice != nil {
			current.choice.index = (current.choice.index + 1) % len(current.choice.options)
		} else if current.text != nil && key.Text != "" {
			*current.text += key.Text
		}
	default:
		if current.text != nil {
			*current.text += key.Text
		}
	}
	f.err = ""
	return false
}

func (f *form) normalKey(key string, slots []slot) bool {
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
