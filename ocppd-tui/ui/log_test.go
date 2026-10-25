package ui

import (
	"strconv"
	"strings"
	"testing"
	"time"
)

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

func Test_given_a_mixed_log_when_not_filtered_then_every_line(t *testing.T) {
	log := []entry{{"CP1", "a"}, {"", "stderr"}}

	lines := filtered(log, "")

	if strings.Join(lines, ",") != "a,stderr" {
		t.Fatalf("got %v", lines)
	}
}
