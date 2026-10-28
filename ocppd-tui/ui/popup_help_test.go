package ui

import (
	"strings"
	"testing"
)

func Test_given_details_open_when_question_mark_then_the_help_lists_the_details_keys(t *testing.T) {
	m := press(charging(), "space")

	m = press(m, "?")

	text := strings.Join(screen(m, 80, 30), "\n")
	if !strings.Contains(text, "next / previous connector") || strings.Contains(text, "spawn stations") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_the_config_tab_when_question_mark_then_the_help_lists_the_config_keys(t *testing.T) {
	m, _ := onConfig()

	m = press(m, "?")

	text := strings.Join(screen(m, 80, 30), "\n")
	if !strings.Contains(text, "flip a true / false value") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_popup_help_open_when_esc_then_the_details_show_again(t *testing.T) {
	m := press(charging(), "space", "?")

	m = press(m, "esc")

	if m.help || m.details == nil {
		t.Fatalf("got help %v, details %v", m.help, m.details != nil)
	}
}

func Test_given_the_config_filter_being_typed_when_question_mark_then_it_is_typed(t *testing.T) {
	m, _ := onConfig()

	m = press(m, "/", "?")

	if m.help {
		t.Fatal("help opened")
	}
}

func Test_given_the_spawn_form_in_insert_mode_when_question_mark_then_no_help_opens(t *testing.T) {
	m := press(charging(), "n")

	m = press(m, "?")

	if m.help {
		t.Fatal("help opened")
	}
}

func Test_given_the_spawn_form_in_normal_mode_when_question_mark_then_the_help_lists_the_form_keys(t *testing.T) {
	m := press(charging(), "n", "esc")

	m = press(m, "?")

	text := strings.Join(screen(m, 80, 30), "\n")
	if !strings.Contains(text, "switch a choice") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_details_open_when_a_count_is_typed_then_the_bar_shows_it(t *testing.T) {
	m := press(charging(), "space")

	m = press(m, "1", "2")

	lines := screen(m, 80, 30)
	if bar := lines[len(lines)-1]; !strings.HasSuffix(bar, " 12") || !strings.Contains(bar, "? help") {
		t.Fatalf("got %q", bar)
	}
}
