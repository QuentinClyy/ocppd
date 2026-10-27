package ui

import (
	"strings"
	"testing"
	"time"
)

func Test_given_a_selected_station_when_space_then_its_details_open(t *testing.T) {
	m := charging()

	m = press(m, "space")

	if m.details == nil || m.details.station != "EVB-0001" {
		t.Fatalf("got %+v", m.details)
	}
}

func Test_given_a_charging_station_when_its_details_are_drawn_then_the_connector_shows_power_soc_meter_and_tx(t *testing.T) {
	m := press(charging(), "space")

	text := strings.Join(screen(m, 100, 30), "\n")

	if !strings.Contains(text, "1   Charging     11.0    47%    0.0 kWh") || !strings.Contains(text, "1042") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_a_connected_station_when_its_details_are_drawn_then_it_is_up_since_then(t *testing.T) {
	m := press(charging(), "space")
	m.zone = time.UTC

	text := strings.Join(screen(m, 100, 30), "\n")

	if !strings.Contains(text, "since") || !strings.Contains(text, "00:00:00") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_details_open_when_j_then_p_then_plug_in_is_sent_for_the_second_connector(t *testing.T) {
	m, sent := recording()
	m = press(m, "space", "j")

	press(m, "p")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"plug_in","connector":2,"station":"CP1"}` {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_details_open_when_a_then_the_id_tag_prompt_targets_the_selected_connector(t *testing.T) {
	m, _ := recording()

	m = press(m, "space", "a")

	if m.prompt == nil || m.prompt.at != (target{station: "CP1", connector: 1}) || m.details == nil {
		t.Fatalf("got prompt %+v, details %+v", m.prompt, m.details)
	}
}

func Test_given_details_open_when_esc_then_they_close(t *testing.T) {
	m := press(charging(), "space")

	m = press(m, "esc")

	if m.details != nil {
		t.Fatalf("got %+v", m.details)
	}
}

func Test_given_details_open_when_the_station_is_removed_then_the_next_key_closes_them(t *testing.T) {
	m := press(charging(), "space")
	m.fleet.Remove("EVB-0001")

	m = press(m, "j")

	if m.details != nil {
		t.Fatalf("got %+v", m.details)
	}
}

func Test_given_a_station_with_events_when_its_details_are_drawn_then_its_last_three_show(t *testing.T) {
	m := press(charging(), "space")

	text := strings.Join(screen(m, 100, 30), "\n")

	if !strings.Contains(text, "meter_sample") || strings.Contains(text, " connected ") {
		t.Fatalf("got\n%s", text)
	}
}
