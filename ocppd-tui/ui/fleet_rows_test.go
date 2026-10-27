package ui

import (
	"fmt"
	"testing"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
)

func fleetOf(lines ...string) fleet.Fleet {
	var f fleet.Fleet
	for _, line := range lines {
		f.Apply(line)
	}
	return f
}

func twoStations() Model {
	m := New(func(string) error { return nil })
	m.fleet = fleetOf(
		`{"station":"CP1","at":1,"kind":"status_changed","connector":1,"status":"Available"}`,
		`{"station":"CP1","at":1,"kind":"status_changed","connector":2,"status":"Available"}`,
		`{"station":"CP2","at":1,"kind":"status_changed","connector":1,"status":"Available"}`,
	)
	return expandAll(m)
}

func expandAll(m Model) Model {
	for _, s := range m.fleet.Stations {
		m.expanded[s.ID] = true
	}
	return m
}

func press(m Model, keys ...string) Model {
	for _, key := range keys {
		next, _ := m.key(keyMsg(key))
		m = next.(Model)
	}
	return m
}

func pressAll(m Model, keys ...string) Model {
	for _, key := range keys {
		next, _ := m.Update(keyMsg(key))
		m = next.(Model)
	}
	return m
}

func keyMsg(key string) tea.KeyPressMsg {
	switch key {
	case "tab":
		return tea.KeyPressMsg{Code: tea.KeyTab}
	case "enter":
		return tea.KeyPressMsg{Code: tea.KeyEnter}
	case "esc":
		return tea.KeyPressMsg{Code: tea.KeyEscape}
	case "backspace":
		return tea.KeyPressMsg{Code: tea.KeyBackspace}
	case "space":
		return tea.KeyPressMsg{Code: tea.KeySpace, Text: " "}
	case "shift+tab":
		return tea.KeyPressMsg{Code: tea.KeyTab, Mod: tea.ModShift}
	case "ctrl+u", "ctrl+d", "ctrl+n":
		return tea.KeyPressMsg{Code: rune(key[5]), Mod: tea.ModCtrl}
	}
	return tea.KeyPressMsg{Code: []rune(key)[0], Text: key}
}

func manyStations(count int) Model {
	m := New(func(string) error { return nil })
	for n := range count {
		m.fleet.Apply(fmt.Sprintf(`{"station":"CP%d","at":1,"kind":"status_changed","connector":1,"status":"Available"}`, n+1))
	}
	return m
}

func Test_given_new_stations_when_listed_then_one_collapsed_row_each(t *testing.T) {
	m := New(func(string) error { return nil })
	m.fleet = twoStations().fleet

	visible := rows(m.fleet, m.expanded)

	if len(visible) != 2 || visible[0] != (row{0, stationRow}) || visible[1] != (row{1, stationRow}) {
		t.Fatalf("got %v", visible)
	}
}

func Test_given_expanded_stations_when_listed_then_their_connectors_follow_them(t *testing.T) {
	m := twoStations()

	visible := rows(m.fleet, m.expanded)

	want := []row{{0, stationRow}, {0, 0}, {0, 1}, {1, stationRow}, {1, 0}}
	if len(visible) != len(want) {
		t.Fatalf("got %v", visible)
	}
	for index := range want {
		if visible[index] != want[index] {
			t.Fatalf("got %v", visible)
		}
	}
}

func Test_given_a_connector_row_when_h_then_the_cursor_goes_to_its_station(t *testing.T) {
	m := press(twoStations(), "j", "j")

	m = press(m, "h")

	if r, _ := m.selected(); r != (row{0, stationRow}) {
		t.Fatalf("got %v", r)
	}
}

func Test_given_a_station_row_when_h_then_it_collapses(t *testing.T) {
	m := twoStations()

	m = press(m, "h")

	if len(rows(m.fleet, m.expanded)) != 3 || m.expanded["CP1"] {
		t.Fatalf("got %v", rows(m.fleet, m.expanded))
	}
}

func Test_given_a_collapsed_station_when_l_then_it_expands_and_the_cursor_stays(t *testing.T) {
	m := press(twoStations(), "h")

	m = press(m, "l")

	if !m.expanded["CP1"] || m.cursor != 0 {
		t.Fatalf("got cursor %d, expanded %v", m.cursor, m.expanded)
	}
}

func Test_given_an_expanded_station_when_l_then_the_cursor_enters_its_first_connector(t *testing.T) {
	m := twoStations()

	m = press(m, "l")

	if r, _ := m.selected(); r != (row{0, 0}) {
		t.Fatalf("got %v", r)
	}
}

func Test_given_the_first_row_when_G_then_the_last_row_and_gg_goes_back(t *testing.T) {
	m := press(twoStations(), "G")

	last, _ := m.selected()
	m = press(m, "g", "g")
	first, _ := m.selected()

	if last != (row{1, 0}) || first != (row{0, stationRow}) {
		t.Fatalf("got last %v, first %v", last, first)
	}
}

func Test_given_the_last_row_when_j_then_the_cursor_stays(t *testing.T) {
	m := press(twoStations(), "G")

	m = press(m, "j")

	if m.cursor != 4 {
		t.Fatalf("got %d", m.cursor)
	}
}

func Test_given_a_collapsed_station_when_enter_then_it_opens_and_the_cursor_stays(t *testing.T) {
	m := New(func(string) error { return nil })
	m.fleet = twoStations().fleet

	m = press(m, "enter")

	if !m.expanded["CP1"] || m.cursor != 0 {
		t.Fatalf("got cursor %d, expanded %v", m.cursor, m.expanded)
	}
}

func Test_given_an_open_station_when_enter_then_it_closes(t *testing.T) {
	m := twoStations()

	m = press(m, "enter")

	if m.expanded["CP1"] {
		t.Fatalf("got %v", m.expanded)
	}
}

func Test_given_a_connector_row_when_enter_then_its_station_closes_and_the_cursor_returns_to_it(t *testing.T) {
	m := press(twoStations(), "j", "j")

	m = press(m, "enter")

	if r, _ := m.selected(); r != (row{0, stationRow}) || m.expanded["CP1"] {
		t.Fatalf("got %v, expanded %v", r, m.expanded)
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
