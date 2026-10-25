package main

import (
	"strings"
	"testing"

	tea "charm.land/bubbletea/v2"
)

func Test_given_more_lines_than_rows_when_rendered_then_the_newest_fill_the_pane(t *testing.T) {
	m := model{csms: "ws://x", width: 20, height: 6, log: []string{"a", "b", "c", "d"}}

	screen := render(m)

	want := " ocppd  ws://x\n" + rule(20) + "\n c\n d\n" + rule(20) + "\n q quit"
	if screen != want {
		t.Fatalf("got\n%s\nwant\n%s", screen, want)
	}
}

func Test_given_fewer_lines_than_rows_when_rendered_then_the_pane_keeps_its_height(t *testing.T) {
	m := model{csms: "ws://x", width: 20, height: 7, log: []string{"a"}}

	screen := render(m)

	if lines := strings.Count(screen, "\n") + 1; lines != 7 {
		t.Fatalf("got %d lines:\n%s", lines, screen)
	}
}

func Test_given_a_line_wider_than_the_terminal_when_rendered_then_it_is_cut_to_the_width(t *testing.T) {
	m := model{csms: "ws://x", width: 10, height: 5, log: []string{"0123456789abcdef"}}

	screen := render(m)

	if !strings.Contains(screen, "\n 012345678\n") {
		t.Fatalf("got\n%s", screen)
	}
}

func Test_given_q_when_pressed_then_the_program_quits(t *testing.T) {
	m := newModel("ws://x")

	_, cmd := m.Update(tea.KeyPressMsg{Code: 'q', Text: "q"})

	if cmd == nil {
		t.Fatal("no command")
	}
	if _, quit := cmd().(tea.QuitMsg); !quit {
		t.Fatalf("got %T", cmd())
	}
}

func rule(width int) string {
	return " " + strings.Repeat("─", width-2)
}
