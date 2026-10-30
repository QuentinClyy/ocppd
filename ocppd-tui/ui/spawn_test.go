package ui

import (
	"strings"
	"testing"

	tea "charm.land/bubbletea/v2"
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

func Test_given_no_station_id_when_spawned_then_the_form_stays_with_an_error_and_nothing_is_sent(t *testing.T) {
	m, sent := recording()

	m = press(m, "n", "enter")

	text := strings.Join(screen(m, 80, 30), "\n")
	if m.form == nil || !strings.Contains(text, "! add at least one station id") || len(*sent) != 0 {
		t.Fatalf("sent %v in\n%s", *sent, text)
	}
}

func Test_given_the_spawn_form_when_esc_twice_then_it_closes_without_sending(t *testing.T) {
	m, sent := recording()

	m = press(m, keys([]string{"n", "tab"}, typed("EVB1"), []string{"esc", "esc"})...)

	if m.form != nil || len(*sent) != 0 {
		t.Fatalf("form %v, sent %v", m.form, *sent)
	}
}

func Test_given_a_typed_password_when_the_form_is_drawn_then_it_shows_as_typed(t *testing.T) {
	m, _ := recording()

	m = press(m, keys([]string{"n", "tab", "tab"}, typed("secret"))...)

	text := ansi.Strip(strings.Join(screen(m, 80, 30), "\n"))
	if !strings.Contains(text, "secret") || strings.Contains(text, "•") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_normal_mode_when_q_then_the_form_closes(t *testing.T) {
	m, _ := recording()

	m = press(m, "n", "esc", "q")

	if m.form != nil {
		t.Fatalf("got %+v", m.form)
	}
}

func Test_given_normal_mode_on_an_id_when_i_then_typing_fills_it(t *testing.T) {
	m, sent := recording()

	press(m, keys([]string{"n", "esc", "j", "i"}, typed("EVB1"), []string{"enter"})...)

	want := `{"command":"spawn","ocpp":"1.6","stations":[{"id":"EVB1"}]}`
	if len(*sent) != 1 || (*sent)[0] != want {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_normal_mode_when_enter_then_the_spawn_is_sent(t *testing.T) {
	m, sent := recording()

	press(m, keys([]string{"n", "tab"}, typed("A"), []string{"esc", "enter"})...)

	if len(*sent) != 1 {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_a_short_terminal_when_the_last_field_is_focused_then_it_shows(t *testing.T) {
	m, _ := recording()
	m = press(sized(m, 80, 12), "n", "esc", "G")

	text := strings.Join(screen(m, 80, 12), "\n")

	if !strings.Contains(text, "csms") || strings.Contains(text, "stations ") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_an_invalid_field_when_spawned_then_the_error_sits_in_the_bottom_border(t *testing.T) {
	m, _ := recording()

	m = press(m, keys([]string{"n", "tab"}, typed("A"), []string{"esc", "G", "k", "k", "k", "k", "k", "i"}, typed("x"), []string{"enter"})...)

	text := strings.Join(screen(m, 80, 30), "\n")
	if !strings.Contains(text, "╰─  INSERT  ! phases must be a number") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_normal_mode_when_drawn_then_the_bottom_border_says_normal(t *testing.T) {
	m, _ := recording()

	m = press(m, "n", "esc")

	if text := strings.Join(screen(m, 80, 30), "\n"); !strings.Contains(text, "╰─  NORMAL ") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_two_choices_when_drawn_then_they_are_one_segmented_control(t *testing.T) {
	m, _ := recording()

	m = press(m, "n")

	if text := strings.Join(screen(m, 80, 30), "\n"); !strings.Contains(text, " list │ prefix + count ") || !strings.Contains(text, " 1.6 │ 2.0.1 ") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_a_typed_id_when_left_twice_and_typing_then_the_text_goes_in_at_the_cursor(t *testing.T) {
	m, sent := recording()

	press(m, keys([]string{"n", "tab"}, typed("EVB1"), []string{"left", "left"}, typed("-0"), []string{"enter"})...)

	want := `{"command":"spawn","ocpp":"1.6","stations":[{"id":"EV-0B1"}]}`
	if len(*sent) != 1 || (*sent)[0] != want {
		t.Fatalf("got %v", *sent)
	}
}

func paste(m Model, msg tea.Msg) Model {
	next, _ := m.Update(msg)
	return next.(Model)
}

func Test_given_the_spawn_csms_field_when_a_url_is_pasted_then_it_is_spawned_with_it(t *testing.T) {
	m, sent := recording()
	m = press(m, keys([]string{"n", "tab"}, typed("A"), []string{"esc", "G", "k", "i"})...)

	m = paste(m, tea.PasteMsg{Content: "ws://csms.example/ocpp\n"})
	press(m, "enter")

	if len(*sent) != 1 || !strings.Contains((*sent)[0], `"csms":"ws://csms.example/ocpp"`) {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_the_spawn_form_in_normal_mode_when_p_is_pressed_then_the_clipboard_is_read_and_pasted(t *testing.T) {
	m, sent := recording()
	m = press(m, keys([]string{"n", "tab"}, typed("A"), []string{"esc"})...)

	_, cmd := m.key(keyMsg("p"))
	m = paste(m, tea.ClipboardMsg{Content: "B"})
	press(m, "enter")

	if cmd == nil || len(*sent) != 1 || !strings.Contains((*sent)[0], `"id":"AB"`) {
		t.Fatalf("cmd %v, got %v", cmd, *sent)
	}
}

func Test_given_the_station_list_when_text_is_pasted_then_nothing_acts_on_it(t *testing.T) {
	m, sent := recording()

	m = paste(m, tea.PasteMsg{Content: "Qn"})

	if m.form != nil || m.confirm != nil || len(*sent) != 0 {
		t.Fatalf("form %v, confirm %v, sent %v", m.form, m.confirm, *sent)
	}
}
