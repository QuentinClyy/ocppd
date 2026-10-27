package ui

import (
	"fmt"
	"strconv"
	"strings"
	"testing"
	"time"

	tea "charm.land/bubbletea/v2"
	"github.com/charmbracelet/x/ansi"
)

func withLog(m Model, count int) Model {
	for n := range count {
		m.log = appendLog(m.log, entry{text: fmt.Sprintf("line %d", n)})
	}
	return m
}

func Test_given_an_event_line_when_logged_then_time_station_kind_and_sorted_fields(t *testing.T) {
	raw := `{"station":"CP1","at":1792893141250,"kind":"status_changed","status":"Charging","connector":1}`

	line := logLine(raw, time.UTC)

	want := "01:52:21  CP1        status_changed         connector=1 status=Charging"
	if line != want {
		t.Fatalf("got  %q\nwant %q", line, want)
	}
}

func Test_given_a_nested_field_when_logged_then_it_shows_as_json(t *testing.T) {
	raw := `{"station":"CP1","at":0,"kind":"intent_rejected","intent":{"command":"stop"}}`

	line := logLine(raw, time.UTC)

	want := `00:00:00  CP1        intent_rejected        intent={"command":"stop"}`
	if line != want {
		t.Fatalf("got  %q\nwant %q", line, want)
	}
}

func Test_given_a_line_that_is_not_json_when_logged_then_kept_as_is(t *testing.T) {
	raw := "not json"

	line := logLine(raw, time.UTC)

	if line != raw {
		t.Fatalf("got %q", line)
	}
}

func Test_given_a_full_log_when_a_line_is_appended_then_the_oldest_is_dropped(t *testing.T) {
	var log []entry
	for n := range logLimit {
		log = append(log, entry{text: strconv.Itoa(n)})
	}

	log = appendLog(log, entry{text: "newest"})

	if len(log) != logLimit || log[0].text != "1" || log[logLimit-1].text != "newest" {
		t.Fatalf("got %d lines from %q to %q", len(log), log[0].text, log[len(log)-1].text)
	}
}

func Test_given_a_mixed_log_when_filtered_by_a_station_then_only_its_lines(t *testing.T) {
	log := []entry{{"CP1", "a"}, {"CP2", "b"}, {"", "stderr"}, {"CP1", "c"}}

	lines := filtered(log, "CP1")

	if strings.Join(lines, ",") != "a,c" {
		t.Fatalf("got %v", lines)
	}
}

func Test_given_an_offset_when_the_tail_is_taken_then_older_lines_show(t *testing.T) {
	lines := []string{"a", "b", "c", "d", "e"}

	shown := tailLines(lines, 2, 2)

	if strings.Join(shown, ",") != " b, c" {
		t.Fatalf("got %q", shown)
	}
}

func Test_given_a_long_log_when_ctrl_u_then_ctrl_d_then_back_at_the_bottom(t *testing.T) {
	m, _ := recording()
	m.width, m.height = 80, 24
	for range 50 {
		m.log = appendLog(m.log, entry{text: "line"})
	}

	up := press(m, "ctrl+u")
	down := press(up, "ctrl+d")

	if up.logOffset != 6 || down.logOffset != 0 {
		t.Fatalf("got up %d, down %d", up.logOffset, down.logOffset)
	}
}

func Test_given_a_mixed_log_when_not_filtered_then_every_line(t *testing.T) {
	log := []entry{{"CP1", "a"}, {"", "stderr"}}

	lines := filtered(log, "")

	if strings.Join(lines, ",") != "a,stderr" {
		t.Fatalf("got %v", lines)
	}
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
