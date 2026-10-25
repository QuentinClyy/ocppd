package ui

import (
	"testing"

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
	m := New("ws://x")
	m.fleet = fleetOf(
		`{"station":"CP1","at":1,"kind":"status_changed","connector":1,"status":"Available"}`,
		`{"station":"CP1","at":1,"kind":"status_changed","connector":2,"status":"Available"}`,
		`{"station":"CP2","at":1,"kind":"status_changed","connector":1,"status":"Available"}`,
	)
	return m
}

func press(m Model, keys ...string) Model {
	for _, key := range keys {
		next, _ := m.key(key)
		m = next.(Model)
	}
	return m
}

func Test_given_new_stations_when_listed_then_expanded_with_their_connectors(t *testing.T) {
	m := twoStations()

	visible := rows(m.fleet, m.collapsed)

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

	if len(rows(m.fleet, m.collapsed)) != 3 || !m.collapsed["CP1"] {
		t.Fatalf("got %v", rows(m.fleet, m.collapsed))
	}
}

func Test_given_a_collapsed_station_when_l_then_it_expands_and_the_cursor_stays(t *testing.T) {
	m := press(twoStations(), "h")

	m = press(m, "l")

	if m.collapsed["CP1"] || m.cursor != 0 {
		t.Fatalf("got cursor %d, collapsed %v", m.cursor, m.collapsed)
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
