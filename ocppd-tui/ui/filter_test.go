package ui

import (
	"strings"
	"testing"
)

func mixedVersions() Model {
	m, _ := recording()
	m.fleet = fleetOf(
		`{"station":"CP1","at":1,"kind":"status_changed","connector":1,"status":"Available"}`,
		`{"station":"CS1","at":1,"kind":"status_changed","evse":1,"connector":1,"status":"Available"}`,
		`{"station":"EVB-7","at":1,"kind":"status_changed","connector":1,"status":"Available"}`,
	)
	return m
}

func Test_given_stations_when_the_filter_is_typed_then_only_ids_containing_it_are_rows(t *testing.T) {
	m := mixedVersions()

	m = press(m, keys([]string{"/"}, typed("evb"), []string{"enter"})...)

	if r, _ := m.selected(); len(m.visibleRows()) != 1 || m.fleet.Stations[r.station].ID != "EVB-7" {
		t.Fatalf("got %v", m.visibleRows())
	}
}

func Test_given_stations_of_both_versions_when_filtered_by_2_0_1_then_only_that_version_shows(t *testing.T) {
	m := mixedVersions()

	m = press(m, keys([]string{"/"}, typed("2.0.1"), []string{"enter"})...)

	if r, _ := m.selected(); len(m.visibleRows()) != 1 || m.fleet.Stations[r.station].ID != "CS1" {
		t.Fatalf("got %v", m.visibleRows())
	}
}

func Test_given_a_station_filter_when_drawn_then_the_header_counts_the_matches(t *testing.T) {
	m := press(mixedVersions(), keys([]string{"/"}, typed("1.6"), []string{"enter"})...)

	if header := screen(m, 100, 24)[0]; !strings.Contains(header, "2 of 3 stations · /1.6") {
		t.Fatalf("got %q", header)
	}
}

func Test_given_a_filter_when_esc_then_it_is_cleared(t *testing.T) {
	m := press(mixedVersions(), keys([]string{"/"}, typed("evb"))...)

	m = press(m, "esc")

	if m.filtering || m.filters[fleetPane].String() != "" || len(m.visibleRows()) != 4 {
		t.Fatalf("filtering %v, rows %v", m.filtering, m.visibleRows())
	}
}

func Test_given_the_log_focused_when_filtered_then_only_matching_lines_and_the_title_shows_it(t *testing.T) {
	m := sized(mixedVersions(), 80, 24)
	m.log = []entry{{text: "boot_status"}, {text: "meter_sample"}, {text: "meter_sample"}}

	m = press(m, keys([]string{"ctrl+j", "/"}, typed("meter"), []string{"enter"})...)

	text := strings.Join(screen(m, 80, 24), "\n")
	if len(m.logLines()) != 2 || !strings.Contains(text, "· /meter") {
		t.Fatalf("got %v in\n%s", m.logLines(), text)
	}
}

func Test_given_a_filtered_log_with_a_raised_cursor_when_a_hidden_line_arrives_then_the_cursor_stays(t *testing.T) {
	m := sized(mixedVersions(), 80, 24)
	for range 40 {
		m.log = appendLog(m.log, entry{text: "meter_sample"})
	}
	m = press(m, keys([]string{"ctrl+j", "/"}, typed("meter"), []string{"enter", "k"})...)

	m = m.addLog(entry{text: "boot_status"})

	if m.logCursor != 1 {
		t.Fatalf("got %d", m.logCursor)
	}
}

func Test_given_a_filter_being_typed_when_drawn_then_the_bar_shows_it(t *testing.T) {
	m := press(mixedVersions(), keys([]string{"/"}, typed("ev"))...)

	if bar := screen(m, 100, 24)[23]; !strings.Contains(bar, "/ ev") || !strings.Contains(bar, "enter keep") {
		t.Fatalf("got %q", bar)
	}
}
