package ui

import (
	"strings"
	"testing"

	tea "charm.land/bubbletea/v2"
	"github.com/charmbracelet/x/ansi"
)

func clickAt(m Model, x, y int) Model {
	next, _ := m.Update(tea.MouseClickMsg{Button: tea.MouseLeft, X: x, Y: y})
	return next.(Model)
}

func dragTo(m Model, x, y int) Model {
	next, _ := m.Update(tea.MouseMotionMsg{Button: tea.MouseLeft, X: x, Y: y})
	return next.(Model)
}

func release(m Model) Model {
	next, _ := m.Update(tea.MouseReleaseMsg{Button: tea.MouseLeft})
	return next.(Model)
}

func collapsed() Model {
	m := sized(New(func(string) error { return nil }), 80, 24)
	m.fleet = twoStations().fleet
	return m
}

func Test_given_two_stations_when_the_second_row_is_clicked_then_it_is_selected(t *testing.T) {
	m := collapsed()
	fleetY, _ := content(m, fleetPane)

	m = clickAt(m, 10, fleetY+1)

	if r, _ := m.selected(); r != (row{1, stationRow}) || m.expanded["CP2"] {
		t.Fatalf("got %v, expanded %v", r, m.expanded)
	}
}

func Test_given_the_selected_row_when_clicked_then_its_station_opens(t *testing.T) {
	m := collapsed()
	fleetY, _ := content(m, fleetPane)

	m = clickAt(m, 10, fleetY)

	if !m.expanded["CP1"] {
		t.Fatalf("got %v", m.expanded)
	}
}

func Test_given_the_fleet_focused_when_the_log_is_clicked_then_the_log_takes_focus(t *testing.T) {
	m := collapsed()
	logY, _ := content(m, logPane)

	m = clickAt(m, 10, logY)

	if m.focus != logPane || m.cursor != 0 {
		t.Fatalf("got focus %v, cursor %d", m.focus, m.cursor)
	}
}

func Test_given_a_wide_screen_when_the_divider_is_dragged_left_then_the_fleet_narrows_to_it(t *testing.T) {
	m := sized(collapsed(), 160, 30)
	divider := m.fleetSize() - 1
	m = clickAt(m, divider, 10)

	m = dragTo(m, 59, 10)

	if m.fleetSize() != 60 {
		t.Fatalf("got %d", m.fleetSize())
	}
}

func Test_given_a_narrow_screen_when_the_rule_is_dragged_down_then_the_fleet_gains_rows(t *testing.T) {
	m := collapsed()
	fleetY, fleetRows := content(m, fleetPane)
	m = clickAt(m, 10, fleetY+fleetRows)

	m = dragTo(m, 10, fleetY+fleetRows+5)

	if m.fleetSize() != fleetRows+5 {
		t.Fatalf("got %d, want %d", m.fleetSize(), fleetRows+5)
	}
}

func Test_given_a_released_drag_when_the_mouse_moves_then_the_split_stays(t *testing.T) {
	m := sized(collapsed(), 160, 30)
	before := m.fleetSize()
	m = release(clickAt(m, before-1, 10))

	m = dragTo(m, 59, 10)

	if m.fleetSize() != before {
		t.Fatalf("got %d, want %d", m.fleetSize(), before)
	}
}

func Test_given_the_help_open_when_a_row_is_clicked_then_the_cursor_stays(t *testing.T) {
	m := press(collapsed(), "?")
	fleetY, _ := content(m, fleetPane)

	m = clickAt(m, 10, fleetY+1)

	if m.cursor != 0 {
		t.Fatalf("got %d", m.cursor)
	}
}

func Test_given_a_wide_screen_when_the_divider_is_dragged_past_the_columns_then_every_row_meets_it(t *testing.T) {
	m := expandAll(sized(collapsed(), 160, 30))
	m = clickAt(m, m.fleetSize()-1, 10)

	m = dragTo(m, 49, 10)

	for index, line := range screen(m, 160, 30)[2:6] {
		if at := strings.Index(line, "│"); ansi.StringWidth(line[:at]) != 49 {
			t.Fatalf("row %d: divider at %d in %q", index, ansi.StringWidth(line[:at]), line)
		}
	}
}

func Test_given_a_long_log_when_a_line_is_clicked_then_the_log_cursor_moves_to_it(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 50)
	logY, logRows := content(m, logPane)

	m = clickAt(m, 10, logY)

	if m.focus != logPane || m.logCursor != logRows-1 {
		t.Fatalf("got focus %v, cursor %d", m.focus, m.logCursor)
	}
}
