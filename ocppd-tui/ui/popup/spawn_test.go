package popup

import (
	"testing"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/charmbracelet/x/ansi"
)

func Test_given_model_fields_when_spawned_then_they_are_in_the_spawn_line(t *testing.T) {
	f := NewSpawn()
	f.stations[0].id = draw.NewEditor("EVB1")
	f.vendor, f.model, f.phases, f.maxCurrent, f.connectors = draw.NewEditor("ACME"), draw.NewEditor("WB22"), draw.NewEditor("1"), draw.NewEditor("16"), draw.NewEditor("2")

	line, err := f.line()

	want := `{"command":"spawn","connectors":2,"max_current_a":16,"model":"WB22","ocpp":"1.6","phases":1,"stations":[{"id":"EVB1"}],"vendor":"ACME"}`
	if err != nil || line != want {
		t.Fatalf("got %q, %v", line, err)
	}
}

func Test_given_phases_that_are_not_a_number_when_spawned_then_an_error_names_the_field(t *testing.T) {
	f := NewSpawn()
	f.stations[0].id = draw.NewEditor("EVB1")
	f.phases = draw.NewEditor("three")

	_, err := f.line()

	if err == nil || err.Error() != "phases must be a number" {
		t.Fatalf("got %v", err)
	}
}

func Test_given_no_station_id_when_enter_then_the_form_stays_open_with_an_error(t *testing.T) {
	f := NewSpawn()

	line, closed := spawnKeys(f, "enter")

	if line != "" || closed || f.err != "add at least one station id" {
		t.Fatalf("got %q, closed %v, err %q", line, closed, f.err)
	}
}

func Test_given_the_form_when_drawn_then_every_line_has_the_same_width(t *testing.T) {
	f := NewSpawn()
	f.err = "add at least one station id"
	frame := f.Frame(40)

	for _, line := range draw.Box(frame.Title, frame.Footer, frame.Lines, formWidth) {
		if ansi.StringWidth(line) != formWidth {
			t.Fatalf("got %d: %q", ansi.StringWidth(line), ansi.Strip(line))
		}
	}
}

func Test_given_the_spawn_form_when_esc_then_j_then_the_next_field_is_focused_and_nothing_typed(t *testing.T) {
	f := NewSpawn()

	spawnKeys(f, "esc", "j")

	if f.focus != 1 || f.stations[0].id.String() != "" {
		t.Fatalf("got focus %d, id %q", f.focus, f.stations[0].id.String())
	}
}

func Test_given_normal_mode_on_the_entry_choice_when_l_then_prefix_mode(t *testing.T) {
	f := NewSpawn()

	spawnKeys(f, "esc", "l")

	if f.isListed() {
		t.Fatal("still listed")
	}
}

func Test_given_normal_mode_when_G_then_gg_then_the_last_then_the_first_field(t *testing.T) {
	f := NewSpawn()
	spawnKeys(f, "esc")
	last := len(f.slots()) - 1

	spawnKeys(f, "G")
	bottom := f.focus
	spawnKeys(f, "g", "g")

	if bottom != last || f.focus != 0 {
		t.Fatalf("got bottom %d (want %d), top %d", bottom, last, f.focus)
	}
}

func Test_given_normal_mode_when_a_count_then_j_then_focus_moves_that_many_fields(t *testing.T) {
	f := NewSpawn()

	spawnKeys(f, "esc", "3", "j")

	if f.focus != 3 {
		t.Fatalf("got %d", f.focus)
	}
}

func Test_given_normal_mode_with_a_pending_count_when_esc_then_the_form_stays_open(t *testing.T) {
	f := NewSpawn()

	_, closed := spawnKeys(f, "esc", "3", "esc")

	if closed || f.pending != "" {
		t.Fatalf("closed %v, pending %q", closed, f.pending)
	}
}
