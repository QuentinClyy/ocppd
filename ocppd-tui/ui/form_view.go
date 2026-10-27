package ui

import "fmt"

const (
	formWidth  = 66
	labelWidth = 12
)

func (f *form) lines() ([]string, int) {
	slots := f.slots()
	f.focus = min(f.focus, len(slots)-1)
	lines := []string{""}
	focused := 0
	for index := 0; index < len(slots); index++ {
		s := slots[index]
		span := 1
		switch s.label {
		case "entry":
			lines = append(lines, section("STATIONS"))
		case "ocpp":
			lines = append(lines, "", section("MODEL"))
		case "csms":
			lines = append(lines, "", section("CONNECTION"))
		case "id":
			span = 2
			if index == 1 {
				lines = append(lines, "", pad("", 2+labelWidth+1)+dim.Render(pad("id", 20)+"password"))
			}
		case "phases":
			span = 3
		}
		if f.focus >= index && f.focus < index+span {
			focused = len(lines)
		}
		lines = append(lines, f.row(slots, index, span))
		if s.label == "entry" && !f.isListed() {
			lines = append(lines, "")
		}
		if s.label == "id" && index+span < len(slots) && slots[index+span].label != "id" {
			lines = append(lines, pad("", 2+labelWidth+1)+accent.Render("+")+dim.Render(" add station"))
		}
		index += span - 1
	}
	return append(lines, ""), focused
}

func (f *form) row(slots []slot, index, span int) string {
	marker := "  "
	if f.focus >= index && f.focus < index+span {
		marker = accent.Render("▌ ")
	}
	name := slots[index].label
	if name == "id" {
		name = fmt.Sprint((index + 1) / 2)
	}
	line := marker + f.label(name, index, labelWidth) + f.value(slots[index], index)
	for next := index + 1; next < index+span; next++ {
		if slots[next].label != "password" {
			line += "  " + f.label(slots[next].label+" ", next, 0)
		} else {
			line += "  "
		}
		line += f.value(slots[next], next)
	}
	return line
}

func (f *form) label(name string, index, width int) string {
	if index == f.focus {
		return accent.Render(pad(name, width))
	}
	return dim.Render(pad(name, width))
}

func (f *form) value(s slot, index int) string {
	focused := index == f.focus
	if s.choice != nil {
		return segmented(*s.choice, focused)
	}
	if focused && !f.normal {
		return editingField(*s.text, s.width)
	}
	return textField(*s.text, s.hint, s.width)
}

func (f *form) footer() string {
	if f.normal {
		return statusLine(normalMode, f.err)
	}
	return statusLine(insertMode, f.err)
}

func (f *form) bar() string {
	keys := [][2]string{{"tab", "next"}, {"⇧tab", "back"}, {"ctrl+n", "add station"}, {"enter", "spawn"}, {"esc", "normal mode"}}
	if f.normal {
		keys = [][2]string{{"j/k", "field"}, {"h/l", "choice"}, {"gg/G", "first/last"}, {"i", "edit"}, {"enter", "spawn"}, {"esc", "cancel"}}
	}
	return keyHints(keys)
}
