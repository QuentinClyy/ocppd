package ui

import (
	"os"
	"strings"
	"testing"

	"github.com/charmbracelet/x/ansi"
)

func recording() (Model, *[]string) {
	var sent []string
	m := New("ws://x", func(line string) error {
		sent = append(sent, line)
		return nil
	})
	m.fleet = fleetOf(
		`{"station":"CP1","at":1,"kind":"status_changed","connector":1,"status":"Available"}`,
		`{"station":"CP1","at":1,"kind":"status_changed","connector":2,"status":"Available"}`,
		`{"station":"CP2","at":1,"kind":"status_changed","connector":1,"status":"Available"}`,
	)
	return expandAll(m), &sent
}

func Test_given_each_connector_key_when_built_then_its_command_for_that_connector(t *testing.T) {
	at := target{station: "CP1", connector: 2}
	want := map[string]string{
		"p": `{"command":"plug_in","connector":2,"station":"CP1"}`,
		"s": `{"command":"stop","connector":2,"station":"CP1"}`,
		"u": `{"command":"unplug","connector":2,"station":"CP1"}`,
		"f": `{"command":"fault","connector":2,"station":"CP1"}`,
		"c": `{"command":"clear_fault","connector":2,"station":"CP1"}`,
	}

	for key, line := range want {
		got, ok := command(key, at)

		if !ok || got != line {
			t.Fatalf("%s: got %q", key, got)
		}
	}
}

func Test_given_x_on_a_station_when_built_then_a_power_cut_for_the_station(t *testing.T) {
	at := target{station: "CP1"}

	line, ok := command("x", at)

	if !ok || line != `{"command":"power_cut","station":"CP1"}` {
		t.Fatalf("got %q", line)
	}
}

func Test_given_a_selected_connector_when_p_then_plug_in_is_sent_for_it(t *testing.T) {
	m, sent := recording()

	press(m, "j", "j", "p")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"plug_in","connector":2,"station":"CP1"}` {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_a_station_with_one_connector_when_p_on_its_row_then_plug_in_for_that_connector(t *testing.T) {
	m, sent := recording()

	press(m, "G", "k", "p")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"plug_in","connector":1,"station":"CP2"}` {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_a_station_with_two_connectors_when_p_on_its_row_then_nothing_is_sent_and_the_cursor_enters_it(t *testing.T) {
	m, sent := recording()

	m = press(m, "p")

	if len(*sent) != 0 {
		t.Fatalf("sent %v", *sent)
	}
	if r, _ := m.selected(); r != (row{0, 0}) {
		t.Fatalf("got %v", r)
	}
}

func Test_given_a_selected_connector_when_an_id_tag_is_typed_and_entered_then_authorize_is_sent(t *testing.T) {
	m, sent := recording()

	press(m, "j", "a", "T", "A", "G", "9", "backspace", "1", "enter")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"authorize","connector":1,"id_tag":"TAG1","station":"CP1"}` {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_the_id_tag_prompt_when_esc_then_nothing_is_sent_and_keys_drive_again(t *testing.T) {
	m, sent := recording()

	m = press(m, "j", "a", "T", "esc", "p")

	if m.prompt != nil || len(*sent) != 1 || !strings.Contains((*sent)[0], "plug_in") {
		t.Fatalf("prompt %v, sent %v", m.prompt, *sent)
	}
}

func Test_given_the_id_tag_prompt_when_q_is_typed_then_it_is_part_of_the_tag(t *testing.T) {
	m, _ := recording()

	m = press(m, "j", "a", "q")

	if m.prompt == nil || string(m.prompt.value) != "q" {
		t.Fatalf("got %+v", m.prompt)
	}
}

func Test_given_help_when_opened_then_every_binding_is_listed(t *testing.T) {
	m, _ := recording()

	m = press(m, "?")

	text := ansi.Strip(strings.Join(screen(m, 80, 30), "\n"))
	for _, b := range bindings {
		if !strings.Contains(text, b.keys) || !strings.Contains(text, b.help) {
			t.Fatalf("missing %q %q in\n%s", b.keys, b.help, text)
		}
	}
}

func Test_given_help_open_when_esc_then_it_closes(t *testing.T) {
	m, _ := recording()

	m = press(m, "?", "esc")

	if m.help {
		t.Fatal("still open")
	}
}

func Test_given_any_width_when_the_key_bar_is_built_then_it_fits_and_ends_with_help(t *testing.T) {
	for _, width := range []int{40, 80, 160} {
		bar := ansi.Strip(keyBar(width))

		if ansi.StringWidth(bar) > width || !strings.HasSuffix(bar, "? help") {
			t.Fatalf("%d: got %q", width, bar)
		}
	}
}

func Test_given_the_help_box_when_drawn_then_every_line_has_the_same_width(t *testing.T) {
	lines := helpLines()

	for _, line := range lines {
		if ansi.StringWidth(line) != helpWidth {
			t.Fatalf("got %d: %q", ansi.StringWidth(line), ansi.Strip(line))
		}
	}
}

func Test_given_a_collapsed_station_with_two_connectors_when_p_then_it_expands_and_nothing_is_sent(t *testing.T) {
	m, sent := recording()
	m.expanded = map[string]bool{}

	m = press(m, "p")

	if len(*sent) != 0 || !m.expanded["CP1"] {
		t.Fatalf("sent %v, expanded %v", *sent, m.expanded)
	}
}

func Test_given_a_selected_station_when_d_then_y_then_it_is_removed_and_remove_is_sent(t *testing.T) {
	m, sent := recording()

	m = press(m, "d", "y")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"remove","station":"CP1"}` || m.fleet.Stations[0].ID != "CP2" {
		t.Fatalf("sent %v, stations %v", *sent, m.fleet.Stations)
	}
}

func Test_given_a_selected_station_when_d_twice_then_it_is_removed_without_the_question(t *testing.T) {
	m, sent := recording()

	m = press(m, "d", "d")

	if len(*sent) != 1 || len(m.fleet.Stations) != 1 || m.removing != "" {
		t.Fatalf("sent %v, stations %v", *sent, m.fleet.Stations)
	}
}

func Test_given_the_remove_question_when_n_then_nothing_is_removed(t *testing.T) {
	m, sent := recording()

	m = press(m, "d", "n")

	if len(*sent) != 0 || len(m.fleet.Stations) != 2 || m.removing != "" {
		t.Fatalf("sent %v, stations %v", *sent, m.fleet.Stations)
	}
}

func Test_given_the_remove_question_when_d_is_pressed_then_the_bar_asks_for_the_station(t *testing.T) {
	m, _ := recording()

	m = press(m, "d")

	bar := screen(m, 80, 24)[23]
	if !strings.Contains(bar, "remove CP1?") || !strings.Contains(bar, "y yes") {
		t.Fatalf("got %q", bar)
	}
}

func Test_given_a_connector_row_when_dd_then_its_station_is_removed(t *testing.T) {
	m, sent := recording()

	press(m, "G", "d", "d")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"remove","station":"CP2"}` {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_the_key_bindings_when_the_tui_doc_is_read_then_it_lists_every_key(t *testing.T) {
	doc, err := os.ReadFile("../../docs/tui.md")
	if err != nil {
		t.Fatal(err)
	}

	for _, b := range bindings {
		for _, key := range strings.Split(b.keys, " / ") {
			if !strings.Contains(string(doc), "`"+key+"`") {
				t.Fatalf("docs/tui.md does not list `%s` (%s)", key, b.help)
			}
		}
	}
}
