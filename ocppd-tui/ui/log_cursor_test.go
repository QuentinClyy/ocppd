package ui

import (
	"fmt"
	"strings"
	"testing"

	tea "charm.land/bubbletea/v2"
)

func focusedLog(count int) Model {
	return press(withLog(sized(twoStations(), 80, 24), count), "ctrl+j")
}

func pressCmd(m Model, keys ...string) (Model, tea.Cmd) {
	var cmd tea.Cmd
	for _, key := range keys {
		var next tea.Model
		next, cmd = m.key(keyMsg(key))
		m = next.(Model)
	}
	return m, cmd
}

func Test_given_a_count_when_k_then_the_log_cursor_moves_that_many_lines(t *testing.T) {
	m := focusedLog(50)

	m = press(m, "5", "k")

	if m.logCursor != 5 {
		t.Fatalf("got %d", m.logCursor)
	}
}

func Test_given_a_count_when_G_in_the_log_then_the_cursor_goes_to_that_line(t *testing.T) {
	m := focusedLog(50)

	m = press(m, "3", "G")

	if m.logLines()[len(m.logLines())-1-m.logCursor] != "line 2" {
		t.Fatalf("got cursor %d", m.logCursor)
	}
}

func Test_given_a_raised_log_cursor_when_a_line_arrives_then_the_cursor_stays_on_its_line(t *testing.T) {
	m := press(focusedLog(50), "k", "k")

	m = m.addLog(entry{text: "newest"})

	if m.logLines()[len(m.logLines())-1-m.logCursor] != "line 47" {
		t.Fatalf("got cursor %d", m.logCursor)
	}
}

func Test_given_the_log_cursor_on_the_newest_line_when_a_line_arrives_then_it_follows_the_tail(t *testing.T) {
	m := focusedLog(50)

	m = m.addLog(entry{text: "newest"})

	if m.logCursor != 0 {
		t.Fatalf("got %d", m.logCursor)
	}
}

func Test_given_a_visual_selection_when_y_then_the_lines_are_copied_oldest_first(t *testing.T) {
	m := press(focusedLog(50), "k", "V", "k", "k")

	m, cmd := pressCmd(m, "y")

	if cmd == nil || fmt.Sprint(cmd()) != "line 46\nline 47\nline 48" || m.visual || m.yanked != 3 {
		t.Fatalf("got visual %v, yanked %d", m.visual, m.yanked)
	}
}

func Test_given_a_visual_selection_when_a_line_arrives_then_the_selection_keeps_its_lines(t *testing.T) {
	m := press(focusedLog(50), "V", "k")

	m = m.addLog(entry{text: "newest"})

	_, cmd := pressCmd(m, "y")
	if cmd == nil || fmt.Sprint(cmd()) != "line 48\nline 49" {
		t.Fatal("selection moved")
	}
}

func Test_given_a_count_when_yy_then_that_many_lines_from_the_cursor_down_are_copied(t *testing.T) {
	m := press(focusedLog(50), "k", "k")

	_, cmd := pressCmd(m, "2", "y", "y")

	if cmd == nil || fmt.Sprint(cmd()) != "line 47\nline 48" {
		t.Fatal("wrong lines")
	}
}

func Test_given_a_visual_selection_when_the_filter_changes_then_it_ends_and_the_cursor_returns_to_the_newest_line(t *testing.T) {
	m := press(focusedLog(50), "V", "k")

	m = press(m, "/", "l", "enter")

	if m.visual || m.logCursor != 0 {
		t.Fatalf("got visual %v, cursor %d", m.visual, m.logCursor)
	}
}

func Test_given_a_yank_when_rendered_then_the_bar_says_how_many_lines(t *testing.T) {
	m := press(focusedLog(50), "V", "k", "y")

	lines := screen(m, 80, 24)

	if !strings.Contains(lines[len(lines)-1], "2 lines yanked") {
		t.Fatalf("got %q", lines[len(lines)-1])
	}
}

func Test_given_visual_mode_when_rendered_then_the_bar_shows_the_mode_and_the_line_count(t *testing.T) {
	m := press(focusedLog(50), "V", "k", "k")

	lines := screen(m, 80, 24)

	bar := lines[len(lines)-1]
	if !strings.Contains(bar, "-- VISUAL LINE --") || !strings.HasSuffix(bar, " 3") {
		t.Fatalf("got %q", bar)
	}
}

func Test_given_visual_mode_when_a_count_is_typed_then_the_bar_shows_the_count(t *testing.T) {
	m := press(focusedLog(50), "V", "k", "1", "2")

	lines := screen(m, 80, 24)

	if bar := lines[len(lines)-1]; !strings.HasSuffix(bar, " 12") {
		t.Fatalf("got %q", bar)
	}
}
