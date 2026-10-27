package popup

import "testing"

const described = `{"station":"CP1","at":1,"kind":"station_described","ocpp":"1.6","vendor":"ACME","model":"WB22","firmware_version":"2.4.0","csms":"ws://csms/CP1","configuration":[` +
	`{"key":"HeartbeatInterval","readonly":false,"value":"300"},` +
	`{"key":"NumberOfConnectors","readonly":true,"value":"2"}]}`

func Test_given_a_popup_on_another_station_when_one_is_described_then_it_keeps_no_description(t *testing.T) {
	p, _ := OpenStation("CP2", 0)

	p, _ = p.Observe(described)

	if p.described.Vendor != "" {
		t.Fatalf("got %+v", p.described)
	}
}

func Test_given_a_described_station_when_j_then_the_description_stays_and_the_next_connector_is_picked(t *testing.T) {
	p, _ := OpenStation("CP1", 0)
	p, _ = p.Observe(described)

	r := stationKeys(p, "j")

	if r.Popup.described.Vendor != "ACME" || r.Popup.connector != 1 {
		t.Fatalf("got %+v", r.Popup)
	}
}

func Test_given_an_edit_when_esc_then_nothing_is_sent_and_the_popup_stays(t *testing.T) {
	p, _ := OpenStation("CP1", 0)
	p, _ = p.Observe(described)

	r := stationKeys(p, "tab", "i", "9", "esc")

	if len(r.Send) != 0 || r.Popup == nil || r.Popup.config.draft != nil {
		t.Fatalf("got %+v", r)
	}
}

func Test_given_a_read_only_key_when_i_then_it_is_not_edited(t *testing.T) {
	p, _ := OpenStation("CP1", 0)
	p, _ = p.Observe(described)

	r := stationKeys(p, "tab", "G", "i")

	if r.Popup.config.draft != nil {
		t.Fatalf("got draft %q", r.Popup.config.draft.String())
	}
}
