package popup

import (
	"fmt"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

const (
	formWidth  = 66
	labelWidth = 12
)

func (f *Spawn) Frame(rows int) draw.Frame {
	lines, focused := f.lines()
	return draw.Frame{Title: draw.Bold.Render("spawn"), Footer: f.footer(), Lines: lines, Top: focused - rows + 1, Width: formWidth}
}

func (f *Spawn) lines() ([]string, int) {
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
				lines = append(lines, "", draw.Pad("", 2+labelWidth+1)+draw.Dim.Render(draw.Pad("id", 20)+"password"))
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
			lines = append(lines, draw.Pad("", 2+labelWidth+1)+draw.Accent.Render("+")+draw.Dim.Render(" add station"))
		}
		index += span - 1
	}
	return append(lines, ""), focused
}

func (f *Spawn) row(slots []slot, index, span int) string {
	marker := "  "
	if f.focus >= index && f.focus < index+span {
		marker = draw.Accent.Render("▌ ")
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

func (f *Spawn) label(name string, index, width int) string {
	if index == f.focus {
		return draw.Accent.Render(draw.Pad(name, width))
	}
	return draw.Dim.Render(draw.Pad(name, width))
}

func (f *Spawn) value(s slot, index int) string {
	focused := index == f.focus
	if s.choice != nil {
		return segmented(*s.choice, focused)
	}
	if focused && !f.normal {
		return draw.EditingField(*s.text, s.width)
	}
	return draw.TextField(s.text.String(), s.hint, s.width)
}

func (f *Spawn) footer() string {
	if f.normal {
		return statusLine(draw.NormalMode, f.err)
	}
	return statusLine(draw.InsertMode, f.err)
}

func (f *Spawn) Bar() string {
	keys := [][2]string{{"tab", "next"}, {"⇧tab", "back"}, {"ctrl+n", "add station"}, {"enter", "spawn"}, {"esc", "normal mode"}}
	if f.normal {
		keys = [][2]string{{"j/k", "field"}, {"h/l", "choice"}, {"gg/G", "first/last"}, {"i", "edit"}, {"enter", "spawn"}, {"esc", "cancel"}}
	}
	return keyHints(keys)
}
