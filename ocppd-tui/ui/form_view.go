package ui

import "strings"

const (
	formWidth  = 58
	labelWidth = 12
	idWidth    = 16
)

func (f *form) lines() ([]string, int) {
	slots := f.slots()
	f.focus = min(f.focus, len(slots)-1)
	lines := []string{""}
	focused := 0
	for index := 0; index < len(slots); index++ {
		s := slots[index]
		switch {
		case s.label == "id":
			if index == 1 {
				lines = append(lines, "", pad("", labelWidth)+dim.Render(pad("id", idWidth)+"password"))
			}
			if f.focus == index || f.focus == index+1 {
				focused = len(lines)
			}
			lines = append(lines, pad("", labelWidth)+pad(f.value(s, index), idWidth)+f.value(slots[index+1], index+1))
			index++
			if index+1 < len(slots) && slots[index+1].label != "id" {
				lines = append(lines, pad("", labelWidth)+accent.Render("ctrl+n")+dim.Render(" add station"), "")
			}
		default:
			if f.focus == index {
				focused = len(lines)
			}
			lines = append(lines, f.label(s, index)+f.value(s, index))
			if s.label == "stations" && !f.isListed() || s.label == "password" {
				lines = append(lines, "")
			}
		}
	}
	lines = append(lines, "")
	if f.err != "" {
		lines = append(lines, warn.Render(f.err))
	}
	return lines, focused
}

func (f *form) label(s slot, index int) string {
	if index == f.focus {
		return accent.Render(pad(s.label, labelWidth))
	}
	return dim.Render(pad(s.label, labelWidth))
}

func (f *form) value(s slot, index int) string {
	focused := index == f.focus
	if s.choice != nil {
		var options []string
		for option, name := range s.choice.options {
			mark := "( ) "
			if option == s.choice.index {
				mark = accent.Render("(•) ")
			}
			options = append(options, mark+name)
		}
		text := strings.Join(options, "   ")
		if focused {
			return text + dim.Render("  ← →")
		}
		return text
	}
	text := *s.text
	if s.secret {
		text = strings.Repeat("•", len([]rune(text)))
	}
	switch {
	case focused && !f.normal:
		return text + cursorBlock.Render(" ")
	case text == "":
		return dim.Render(s.hint)
	}
	return text
}

func (f *form) bar() string {
	keys := [][2]string{{"tab", "next"}, {"⇧tab", "back"}, {"ctrl+n", "add station"}, {"enter", "spawn"}, {"esc", "normal mode"}}
	if f.normal {
		keys = [][2]string{{"j/k", "field"}, {"h/l", "choice"}, {"gg/G", "first/last"}, {"i", "edit"}, {"enter", "spawn"}, {"esc", "cancel"}}
	}
	var items []string
	for _, key := range keys {
		items = append(items, accent.Render(key[0])+" "+key[1])
	}
	return " " + strings.Join(items, "  ")
}
