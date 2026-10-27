package draw

import (
	"testing"

	tea "charm.land/bubbletea/v2"
	"github.com/charmbracelet/x/ansi"
)

func keyMsg(key string) tea.KeyPressMsg {
	codes := map[string]rune{"left": tea.KeyLeft, "right": tea.KeyRight, "home": tea.KeyHome, "delete": tea.KeyDelete, "backspace": tea.KeyBackspace}
	if code, ok := codes[key]; ok {
		return tea.KeyPressMsg{Code: code}
	}
	return tea.KeyPressMsg{Code: []rune(key)[0], Text: key}
}

func editKeys(e Editor, keys ...string) Editor {
	for _, key := range keys {
		e = e.Edit(keyMsg(key))
	}
	return e
}

func Test_given_text_when_left_twice_then_typing_inserts_before_the_last_two_characters(t *testing.T) {
	e := NewEditor("EVB-01")

	e = editKeys(e, "left", "left", "X")

	if e.String() != "EVB-X01" {
		t.Fatalf("got %q", e.String())
	}
}

func Test_given_the_cursor_inside_when_backspace_then_the_character_before_it_goes(t *testing.T) {
	e := NewEditor("abc")

	e = editKeys(e, "left", "backspace")

	if e.String() != "ac" || e.cursor != 1 {
		t.Fatalf("got %q at %d", e.String(), e.cursor)
	}
}

func Test_given_the_cursor_at_the_start_when_delete_then_the_first_character_goes(t *testing.T) {
	e := NewEditor("abc")

	e = editKeys(e, "home", "delete")

	if e.String() != "bc" || e.cursor != 0 {
		t.Fatalf("got %q at %d", e.String(), e.cursor)
	}
}

func Test_given_the_cursor_at_the_end_when_right_then_it_stays(t *testing.T) {
	e := NewEditor("ab")

	e = editKeys(e, "right")

	if e.cursor != 2 {
		t.Fatalf("got %d", e.cursor)
	}
}

func Test_given_a_copy_when_the_original_is_edited_then_the_copy_keeps_its_text(t *testing.T) {
	original := editKeys(NewEditor("abc"), "left")
	copied := original

	original.Edit(keyMsg("X"))

	if copied.String() != "abc" {
		t.Fatalf("got %q", copied.String())
	}
}

func Test_given_text_longer_than_the_view_when_drawn_then_the_cursor_stays_visible(t *testing.T) {
	e := NewEditor("0123456789")

	shown := ansi.Strip(e.View(4))

	if shown != "789 " {
		t.Fatalf("got %q", shown)
	}
}
