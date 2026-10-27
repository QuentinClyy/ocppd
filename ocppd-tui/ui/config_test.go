package ui

import (
	"strings"
	"testing"
)

const configured = `{"station":"CP1","at":1,"kind":"station_described","ocpp":"1.6","vendor":"ocppd","model":"sim","firmware_version":"0.3.1","csms":"ws://csms/CP1","configuration":[` +
	`{"key":"AuthorizationKey","readonly":false},` +
	`{"key":"HeartbeatInterval","readonly":false,"value":"300"},` +
	`{"key":"LocalAuthorizeOffline","readonly":false,"value":"true"},` +
	`{"key":"NumberOfConnectors","readonly":true,"value":"2"}]}`

func onConfig() (Model, *[]string) {
	m, sent := recording()
	m = receive(press(m, "space"), configured)
	m = press(m, "tab")
	*sent = nil
	return m, sent
}

func Test_given_details_open_when_tab_then_the_config_tab_lists_every_key(t *testing.T) {
	m, _ := onConfig()

	text := strings.Join(screen(m, 100, 30), "\n")

	for _, want := range []string{"config", "HeartbeatInterval", "300", "[x] true", "write-only", "2", "ro"} {
		if !strings.Contains(text, want) {
			t.Fatalf("no %q in\n%s", want, text)
		}
	}
}

func Test_given_the_config_tab_when_a_value_is_edited_then_configure_is_sent(t *testing.T) {
	m, sent := onConfig()

	press(m, keys([]string{"j", "i", "backspace", "backspace", "backspace"}, typed("60"), []string{"enter"})...)

	want := `{"command":"configure","key":"HeartbeatInterval","station":"CP1","value":"60"}`
	if len(*sent) != 1 || (*sent)[0] != want {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_an_edit_when_esc_then_nothing_is_sent_and_the_popup_stays(t *testing.T) {
	m, sent := onConfig()

	m = press(m, keys([]string{"j", "i"}, typed("9"), []string{"esc"})...)

	if len(*sent) != 0 || m.details == nil || m.details.config.draft != nil {
		t.Fatalf("sent %v, details %+v", *sent, m.details)
	}
}

func Test_given_a_boolean_key_when_space_then_its_opposite_is_configured(t *testing.T) {
	m, sent := onConfig()

	press(m, "j", "j", "space")

	want := `{"command":"configure","key":"LocalAuthorizeOffline","station":"CP1","value":"false"}`
	if len(*sent) != 1 || (*sent)[0] != want {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_a_read_only_key_when_i_then_it_is_not_edited(t *testing.T) {
	m, _ := onConfig()

	m = press(m, "G", "i")

	if m.details.config.draft != nil {
		t.Fatalf("got draft %q", *m.details.config.draft)
	}
}

func Test_given_a_filter_when_typed_then_only_matching_keys_show(t *testing.T) {
	m, _ := onConfig()

	m = press(m, keys([]string{"/"}, typed("heart"), []string{"enter"})...)

	text := strings.Join(screen(m, 100, 30), "\n")
	if !strings.Contains(text, "HeartbeatInterval") || strings.Contains(text, "NumberOfConnectors") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_the_config_tab_when_the_key_changes_then_its_new_value_shows(t *testing.T) {
	m, _ := onConfig()

	m = receive(m, `{"station":"CP1","at":2,"kind":"configuration_changed","key":"HeartbeatInterval","value":"60"}`)

	if text := strings.Join(screen(m, 100, 30), "\n"); !strings.Contains(text, " 60 ") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_the_config_tab_when_a_change_is_refused_then_the_reason_shows_in_the_border(t *testing.T) {
	m, _ := onConfig()

	m = receive(m, `{"station":"CP1","at":2,"kind":"intent_rejected","reason":"HeartbeatInterval refused (Rejected)"}`)

	if text := strings.Join(screen(m, 100, 30), "\n"); !strings.Contains(text, "! HeartbeatInterval refused (Rejected)") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_a_variable_with_an_instance_when_it_changes_then_the_station_is_described_again(t *testing.T) {
	m, sent := onConfig()

	receive(m, `{"station":"CP1","at":2,"kind":"variable_changed","component":"EVSE","variable":"Power","value":"7"}`)

	if len(*sent) != 1 || (*sent)[0] != `{"command":"describe","station":"CP1"}` {
		t.Fatalf("got %v", *sent)
	}
}
