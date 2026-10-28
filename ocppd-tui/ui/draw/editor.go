package draw

import (
	"slices"
	"strings"

	tea "charm.land/bubbletea/v2"
)

type Editor struct {
	runes  []rune
	cursor int
}

func NewEditor(text string) Editor {
	runes := []rune(text)
	return Editor{runes: runes, cursor: len(runes)}
}

func (e Editor) String() string {
	return string(e.runes)
}

func (e Editor) Edit(key tea.KeyPressMsg) Editor {
	switch key.String() {
	case "left":
		e.cursor = max(e.cursor-1, 0)
	case "right":
		e.cursor = min(e.cursor+1, len(e.runes))
	case "home", "ctrl+a":
		e.cursor = 0
	case "end", "ctrl+e":
		e.cursor = len(e.runes)
	case "backspace":
		if e.cursor > 0 {
			e.runes = slices.Delete(slices.Clone(e.runes), e.cursor-1, e.cursor)
			e.cursor--
		}
	case "delete":
		if e.cursor < len(e.runes) {
			e.runes = slices.Delete(slices.Clone(e.runes), e.cursor, e.cursor+1)
		}
	default:
		typed := []rune(key.Text)
		e.runes = slices.Insert(slices.Clone(e.runes), e.cursor, typed...)
		e.cursor += len(typed)
	}
	return e
}

func (e Editor) Paste(text string) Editor {
	pasted := []rune(strings.NewReplacer("\r", "", "\n", "").Replace(strings.TrimSpace(text)))
	e.runes = slices.Insert(slices.Clone(e.runes), e.cursor, pasted...)
	e.cursor += len(pasted)
	return e
}

func (e Editor) Window(width int) (before, under, after string) {
	start := max(e.cursor-(width-1), 0)
	before, under = string(e.runes[start:e.cursor]), " "
	if e.cursor < len(e.runes) {
		under = string(e.runes[e.cursor])
		after = string(e.runes[e.cursor+1 : min(len(e.runes), start+width)])
	}
	return before, under, after
}

func (e Editor) View(width int) string {
	before, under, after := e.Window(width)
	return before + CursorBlock.Render(under) + after
}
