package ui

import (
	"strings"
	"testing"

	"github.com/charmbracelet/x/ansi"
)

func typed(text string) []string {
	var keys []string
	for _, r := range text {
		keys = append(keys, string(r))
	}
	return keys
}

func keys(groups ...[]string) []string {
	var all []string
	for _, group := range groups {
		all = append(all, group...)
	}
	return all
}

func Test_given_the_spawn_form_when_an_id_and_password_are_entered_then_a_listed_spawn_is_sent(t *testing.T) {
	m, sent := recording()

	m = press(m, keys([]string{"n", "tab"}, typed("EVB1"), []string{"tab"}, typed("secret"), []string{"enter"})...)

	want := `{"command":"spawn","ocpp":"1.6","stations":[{"id":"EVB1","password":"secret"}]}`
	if m.form != nil || len(*sent) != 1 || (*sent)[0] != want {
		t.Fatalf("form %v, sent %v", m.form, *sent)
	}
}

func Test_given_two_stations_added_with_ctrl_n_when_spawned_then_both_are_listed(t *testing.T) {
	m, sent := recording()

	press(m, keys([]string{"n", "tab"}, typed("A"), []string{"ctrl+n"}, typed("B"), []string{"enter"})...)

	want := `{"command":"spawn","ocpp":"1.6","stations":[{"id":"A"},{"id":"B"}]}`
	if len(*sent) != 1 || (*sent)[0] != want {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_prefix_mode_and_2_0_1_when_spawned_then_a_numbered_spawn_is_sent(t *testing.T) {
	m, sent := recording()

	press(m, keys([]string{"n", "space", "tab"}, typed("CS"), []string{"tab"}, typed("3"), []string{"tab", "tab", "right", "enter"})...)

	want := `{"command":"spawn","count":3,"ocpp":"2.0.1","prefix":"CS"}`
	if len(*sent) != 1 || (*sent)[0] != want {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_prefix_mode_left_empty_when_spawned_then_one_cp_station(t *testing.T) {
	m, sent := recording()

	press(m, "n", "space", "enter")

	want := `{"command":"spawn","count":1,"ocpp":"1.6","prefix":"CP"}`
	if len(*sent) != 1 || (*sent)[0] != want {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_model_fields_when_spawned_then_they_are_in_the_spawn_line(t *testing.T) {
	f := newForm()
	f.stations[0].id = "EVB1"
	f.vendor, f.model, f.phases, f.maxCurrent, f.connectors = "ACME", "WB22", "1", "16", "2"

	line, err := f.line()

	want := `{"command":"spawn","connectors":2,"max_current_a":16,"model":"WB22","ocpp":"1.6","phases":1,"stations":[{"id":"EVB1"}],"vendor":"ACME"}`
	if err != nil || line != want {
		t.Fatalf("got %q, %v", line, err)
	}
}

func Test_given_no_station_id_when_spawned_then_the_form_stays_with_an_error_and_nothing_is_sent(t *testing.T) {
	m, sent := recording()

	m = press(m, "n", "enter")

	if m.form == nil || m.form.err != "add at least one station id" || len(*sent) != 0 {
		t.Fatalf("form %+v, sent %v", m.form, *sent)
	}
}

func Test_given_phases_that_are_not_a_number_when_spawned_then_an_error_names_the_field(t *testing.T) {
	f := newForm()
	f.stations[0].id = "EVB1"
	f.phases = "three"

	_, err := f.line()

	if err == nil || err.Error() != "phases must be a number" {
		t.Fatalf("got %v", err)
	}
}

func Test_given_the_spawn_form_when_esc_then_it_closes_without_sending(t *testing.T) {
	m, sent := recording()

	m = press(m, keys([]string{"n", "tab"}, typed("EVB1"), []string{"esc"})...)

	if m.form != nil || len(*sent) != 0 {
		t.Fatalf("form %v, sent %v", m.form, *sent)
	}
}

func Test_given_a_typed_password_when_the_form_is_drawn_then_only_dots_show(t *testing.T) {
	m, _ := recording()

	m = press(m, keys([]string{"n", "tab", "tab"}, typed("secret"))...)

	text := ansi.Strip(strings.Join(screen(m, 80, 30), "\n"))
	if strings.Contains(text, "secret") || !strings.Contains(text, "••••••") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_the_form_when_drawn_then_every_line_has_the_same_width(t *testing.T) {
	f := newForm()
	f.err = "add at least one station id"

	for _, line := range f.lines() {
		if ansi.StringWidth(line) != formWidth {
			t.Fatalf("got %d: %q", ansi.StringWidth(line), ansi.Strip(line))
		}
	}
}
