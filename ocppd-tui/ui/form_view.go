package ui

import (
	"fmt"
	"strings"
)

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
	text := tail(*s.text, s.width-2)
	switch {
	case focused && !f.normal:
		return field.Render(" "+text) + cursorBlock.Render(" ") + field.Render(pad("", s.width-2-len([]rune(text))))
	case text == "":
		return fieldHint.Render(pad(" "+s.hint, s.width))
	}
	return field.Render(pad(" "+text, s.width))
}

func segmented(c choice, focused bool) string {
	parts := make([]string, len(c.options))
	for index, option := range c.options {
		style := dim
		switch {
		case index == c.index && focused:
			style = chosen
		case index == c.index:
			style = selected
		}
		parts[index] = style.Render(" " + option + " ")
	}
	return strings.Join(parts, dim.Render("│"))
}

func section(title string) string {
	return "  " + heading.Render(title)
}

func (f *form) footer() string {
	mode := insertBadge.Render(" INSERT ")
	if f.normal {
		mode = normalBadge.Render(" NORMAL ")
	}
	if f.err == "" {
		return mode
	}
	return mode + " " + warn.Render("! "+f.err)
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
