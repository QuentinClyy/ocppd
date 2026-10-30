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
		case "voltage V":
			span = 3
		case "connectors":
			span = 2
		}
		if f.focus >= index && f.focus < index+span {
			focused = len(lines)
		}
		lines = append(lines, f.row(slots, index, span))
		if s.label == "voltage V" {
			lines = append(lines, draw.Pad("", 2+labelWidth)+draw.Dim.Render(f.rating()))
		}
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
	name := slots[index].label
	if name == "id" {
		name = fmt.Sprint((index + 1) / 2)
	}
	line := f.marker(index, span) + f.label(name, index, labelWidth) + f.value(slots[index], index)
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

func (f *Spawn) Bindings() []draw.Binding {
	return spawnBindings
}

func (f *Spawn) Bar(width int) string {
	if f.normal {
		return draw.KeyBar(width, spawnBindings, f.pending)
	}
	return keyHints([][2]string{{"tab", "next"}, {"⇧tab", "back"}, {"ctrl+n", "add station"}, {"enter", "spawn"}, {"esc", "normal mode"}})
}
