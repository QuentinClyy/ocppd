package ui

import (
	"fmt"
	"strings"
	"testing"

	tea "charm.land/bubbletea/v2"
	"github.com/charmbracelet/x/ansi"
)

func sized(m Model, width, height int) Model {
	m.width, m.height = width, height
	return m
}

func withLog(m Model, count int) Model {
	for n := range count {
		m.log = appendLog(m.log, entry{text: fmt.Sprintf("line %d", n)})
	}
	return m
}

func manyStations(count int) Model {
	m := New(func(string) error { return nil })
	for n := range count {
		m.fleet.Apply(fmt.Sprintf(`{"station":"CP%d","at":1,"kind":"status_changed","connector":1,"status":"Available"}`, n+1))
	}
	return m
}

func wheelAt(m Model, button tea.MouseButton, x, y, times int) Model {
	for range times {
		next, _ := m.Update(tea.MouseWheelMsg{Button: button, X: x, Y: y})
		m = next.(Model)
	}
	return m
}

func Test_given_a_long_log_when_the_wheel_scrolls_up_over_it_then_older_lines_by_three(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 50)
	logY := m.layout().logY

	m = wheelAt(m, tea.MouseWheelUp, 10, logY, 1)

	if m.logOffset != 3 {
		t.Fatalf("got %d", m.logOffset)
	}
}

func Test_given_a_long_log_when_the_wheel_scrolls_far_up_then_the_first_line_tops_a_full_pane(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 50)
	l := m.layout()

	m = wheelAt(m, tea.MouseWheelUp, 10, l.logY, 100)

	if m.logOffset != 50-l.logRows {
		t.Fatalf("got offset %d, want %d", m.logOffset, 50-l.logRows)
	}
	lines := strings.Split(ansi.Strip(render(m)), "\n")
	if !strings.HasPrefix(lines[l.logY], " line 0") || !strings.HasPrefix(lines[l.logY+l.logRows-1], fmt.Sprintf(" line %d", l.logRows-1)) {
		t.Fatalf("got %q .. %q", lines[l.logY], lines[l.logY+l.logRows-1])
	}
}

func Test_given_the_bottom_of_the_log_when_the_wheel_scrolls_down_then_it_stays_at_the_bottom(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 50)

	m = wheelAt(m, tea.MouseWheelDown, 10, m.layout().logY, 1)

	if m.logOffset != 0 {
		t.Fatalf("got %d", m.logOffset)
	}
}

func Test_given_a_scrolled_up_log_when_a_line_arrives_then_the_view_stays_on_the_same_lines(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 50)
	m = wheelAt(m, tea.MouseWheelUp, 10, m.layout().logY, 1)

	m = m.addLog(entry{text: "newest"})

	if m.logOffset != 4 {
		t.Fatalf("got %d", m.logOffset)
	}
}

func Test_given_the_log_at_its_bottom_when_a_line_arrives_then_it_follows_the_tail(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 50)

	m = m.addLog(entry{text: "newest"})

	if m.logOffset != 0 {
		t.Fatalf("got %d", m.logOffset)
	}
}

func Test_given_more_stations_than_rows_when_the_wheel_scrolls_down_over_them_then_the_view_moves_and_drags_the_cursor(t *testing.T) {
	m := sized(manyStations(30), 80, 24)

	m = wheelAt(m, tea.MouseWheelDown, 10, m.layout().fleetY, 2)

	if m.fleetTop() != 6 || m.cursor != 6 {
		t.Fatalf("got top %d, cursor %d", m.fleetTop(), m.cursor)
	}
}

func Test_given_more_stations_than_rows_when_the_wheel_scrolls_far_down_then_the_last_row_ends_the_pane(t *testing.T) {
	m := sized(manyStations(30), 80, 24)
	l := m.layout()

	m = wheelAt(m, tea.MouseWheelDown, 10, l.fleetY, 100)

	if m.fleetTop() != 30-l.fleetRows {
		t.Fatalf("got top %d, want %d", m.fleetTop(), 30-l.fleetRows)
	}
}

func Test_given_more_stations_than_rows_when_j_passes_the_bottom_then_the_view_follows_and_stays_when_k(t *testing.T) {
	m := sized(manyStations(30), 80, 24)
	rowsShown := m.layout().fleetRows
	keys := make([]string, rowsShown)
	for index := range keys {
		keys[index] = "j"
	}

	down := pressAll(m, keys...)
	up := pressAll(down, "k")

	if down.fleetTop() != 1 || up.fleetTop() != 1 {
		t.Fatalf("got down %d, up %d", down.fleetTop(), up.fleetTop())
	}
}

func Test_given_a_wide_screen_when_the_wheel_scrolls_left_of_the_divider_then_the_fleet_scrolls(t *testing.T) {
	m := withLog(sized(manyStations(40), 160, 30), 50)

	m = wheelAt(m, tea.MouseWheelDown, 10, 10, 1)

	if m.fleetTop() != 3 || m.logOffset != 0 {
		t.Fatalf("got top %d, offset %d", m.fleetTop(), m.logOffset)
	}
}

func Test_given_a_wide_screen_when_the_wheel_scrolls_right_of_the_divider_then_the_log_scrolls(t *testing.T) {
	m := withLog(sized(manyStations(40), 160, 30), 50)

	m = wheelAt(m, tea.MouseWheelUp, 120, 10, 1)

	if m.fleetTop() != 0 || m.logOffset != 3 {
		t.Fatalf("got top %d, offset %d", m.fleetTop(), m.logOffset)
	}
}

func Test_given_lines_that_fit_when_the_thumb_is_placed_then_there_is_none(t *testing.T) {
	start, end := thumb(5, 10, 0)

	if start != end {
		t.Fatalf("got %d..%d", start, end)
	}
}

func Test_given_a_list_twice_the_pane_when_at_the_top_then_a_half_pane_thumb_at_the_top(t *testing.T) {
	start, end := thumb(20, 10, 0)

	if start != 0 || end != 5 {
		t.Fatalf("got %d..%d", start, end)
	}
}

func Test_given_a_list_twice_the_pane_when_at_the_end_then_the_thumb_ends_the_pane(t *testing.T) {
	start, end := thumb(20, 10, 10)

	if start != 5 || end != 10 {
		t.Fatalf("got %d..%d", start, end)
	}
}

func Test_given_a_long_log_following_its_tail_when_rendered_then_the_thumb_is_at_the_bottom_of_the_log_pane(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 200)
	l := m.layout()

	lines := strings.Split(ansi.Strip(render(m)), "\n")

	last, first := lines[l.logY+l.logRows-1], lines[l.logY]
	if !strings.HasSuffix(last, "▐") || strings.HasSuffix(first, "▐") {
		t.Fatalf("got first %q, last %q", first, last)
	}
}

func Test_given_a_short_log_when_rendered_then_no_scrollbar(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 2)

	text := ansi.Strip(render(m))

	if strings.Contains(text, "▐") {
		t.Fatalf("got\n%s", text)
	}
}
