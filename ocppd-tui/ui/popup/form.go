package popup

import (
	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/motion"
)

type slot struct {
	label  string
	text   *draw.Editor
	choice *choice
	hint   string
	width  int
}

type fields struct {
	focus   int
	normal  bool
	pending string
	err     string
}

func (f *fields) submit(msg tea.KeyPressMsg, slots []slot, extra func(string), line func() (string, error)) (string, bool) {
	key := msg.String()
	switch {
	case key == "esc" && !f.normal:
		f.normal = true
		return "", false
	case f.normal && f.pending == "" && (key == "esc" || key == "q"):
		return "", true
	}
	if !f.edit(msg, slots, extra) {
		return "", false
	}
	text, err := line()
	if err != nil {
		f.err = err.Error()
		return "", false
	}
	return text, true
}

func (f *fields) paste(slots []slot, text string) {
	current := slots[min(f.focus, len(slots)-1)]
	if current.text != nil {
		*current.text = current.text.Paste(text)
		f.err = ""
	}
}

func (f *fields) pastes(key string) bool {
	return f.normal && f.pending == "" && key == "p"
}

func (f *fields) edit(key tea.KeyPressMsg, slots []slot, extra func(string)) bool {
	f.focus = min(f.focus, len(slots)-1)
	if f.normal {
		return f.normalKey(key.String(), slots, extra)
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
		extra("ctrl+n")
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

func (f *fields) normalKey(key string, slots []slot, extra func(string)) bool {
	pending, count, key := motion.Feed(f.pending, key, "g")
	f.pending = pending
	current := slots[f.focus]
	switch key {
	case "tab":
		key = "j"
	case "shift+tab":
		key = "k"
	}
	if index, ok := motion.Move(key, count, f.focus, len(slots)-1); ok {
		f.focus = index
		return false
	}
	switch key {
	case "enter":
		return true
	case "h", "l", "left", "right":
		if current.choice != nil {
			current.choice.index = (current.choice.index + 1) % len(current.choice.options)
		}
	case "i", "a":
		f.normal = false
	case "ctrl+n":
		extra("ctrl+n")
	}
	f.err = ""
	return false
}

func (f *fields) label(name string, index, width int) string {
	if index == f.focus {
		return draw.Accent.Render(draw.Pad(name, width))
	}
	return draw.Dim.Render(draw.Pad(name, width))
}

func (f *fields) value(s slot, index int) string {
	focused := index == f.focus
	if s.choice != nil {
		return segmented(*s.choice, focused)
	}
	if focused && !f.normal {
		return draw.EditingField(*s.text, s.width)
	}
	return draw.TextField(s.text.String(), s.hint, s.width)
}

func (f *fields) marker(index, span int) string {
	if f.focus >= index && f.focus < index+span {
		return draw.Accent.Render("▌ ")
	}
	return "  "
}

func (f *fields) footer() string {
	if f.normal {
		return statusLine(draw.NormalMode, f.err)
	}
	return statusLine(draw.InsertMode, f.err)
}
