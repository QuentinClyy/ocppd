package ui

import (
	"os"
	"strings"
	"testing"
)

func recording() (Model, *[]string) {
	var sent []string
	m := New(func(line string) error {
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

func Test_given_a_collapsed_station_with_two_connectors_when_p_then_it_expands_and_nothing_is_sent(t *testing.T) {
	m, sent := recording()
	m.expanded = map[string]bool{}

	m = press(m, "p")

	if len(*sent) != 0 || !m.expanded["CP1"] {
		t.Fatalf("sent %v, expanded %v", *sent, m.expanded)
	}
}

func Test_given_the_key_bindings_when_the_tui_doc_is_read_then_it_lists_every_key(t *testing.T) {
	doc, err := os.ReadFile("../../docs/tui.md")
	if err != nil {
		t.Fatal(err)
	}

	for _, b := range append(bindingsFor(fleetPane), paneBindings[logPane]...) {
		for _, key := range strings.Split(b.Keys, " / ") {
			if !strings.Contains(string(doc), "`"+key+"`") {
				t.Fatalf("docs/tui.md does not list `%s` (%s)", key, b.Help)
			}
		}
	}
}
