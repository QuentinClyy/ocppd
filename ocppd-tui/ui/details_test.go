package ui

import (
	"strings"
	"testing"
	"time"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ocppd"
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

	if last := (*sent)[len(*sent)-1]; last != `{"command":"plug_in","connector":2,"station":"CP1"}` {
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

const described = `{"station":"CP1","at":1,"kind":"station_described","ocpp":"1.6","vendor":"ACME","model":"WB22","firmware_version":"2.4.0","csms":"ws://csms/CP1","configuration":[]}`

func receive(m Model, line string) Model {
	next, _ := m.Update(ocppd.EventMsg(line))
	return next.(Model)
}

func Test_given_a_selected_station_when_space_then_describe_is_asked_for_it(t *testing.T) {
	m, sent := recording()

	press(m, "space")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"describe","station":"CP1"}` {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_details_open_when_the_station_is_described_then_its_vendor_model_firmware_and_csms_show(t *testing.T) {
	m, _ := recording()
	m = press(m, "space")

	m = receive(m, described)

	text := strings.Join(screen(m, 100, 30), "\n")
	for _, want := range []string{"ACME", "WB22", "2.4.0", "ws://csms/CP1"} {
		if !strings.Contains(text, want) {
			t.Fatalf("no %q in\n%s", want, text)
		}
	}
}

func Test_given_details_open_on_another_station_when_one_is_described_then_nothing_changes(t *testing.T) {
	m, _ := recording()
	m = press(m, "G", "space")

	m = receive(m, described)

	if m.details.described.Vendor != "" {
		t.Fatalf("got %+v", m.details)
	}
}

func Test_given_a_description_when_received_then_it_is_not_logged(t *testing.T) {
	m, _ := recording()

	m = receive(m, described)

	if len(m.log) != 0 {
		t.Fatalf("got %v", m.log)
	}
}

func Test_given_a_described_station_when_j_then_the_description_stays(t *testing.T) {
	m, _ := recording()
	m = receive(press(m, "space"), described)

	m = press(m, "j")

	if m.details.described.Vendor != "ACME" || m.details.connector != 1 {
		t.Fatalf("got %+v", m.details)
	}
}
