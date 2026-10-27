package ui

import (
	"strings"
	"testing"
)

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

	if len(*sent) != 1 || len(m.fleet.Stations) != 1 || m.confirm != nil {
		t.Fatalf("sent %v, stations %v", *sent, m.fleet.Stations)
	}
}

func Test_given_the_remove_question_when_n_then_nothing_is_removed(t *testing.T) {
	m, sent := recording()

	m = press(m, "d", "n")

	if len(*sent) != 0 || len(m.fleet.Stations) != 2 || m.confirm != nil {
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

func Test_given_Q_then_y_when_pressed_then_shutdown_is_sent(t *testing.T) {
	m, sent := recording()

	m = press(m, "Q", "y")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"shutdown"}` || !m.stopping {
		t.Fatalf("sent %v, stopping %v", *sent, m.stopping)
	}
}

func Test_given_Q_then_n_when_pressed_then_nothing_is_sent(t *testing.T) {
	m, sent := recording()

	press(m, "Q", "n")

	if len(*sent) != 0 {
		t.Fatalf("got %v", *sent)
	}
}
