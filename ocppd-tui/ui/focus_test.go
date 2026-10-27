package ui

import (
	"strings"
	"testing"

	"github.com/charmbracelet/x/ansi"
)

func Test_given_a_wide_screen_when_ctrl_l_then_the_log_takes_focus(t *testing.T) {
	m := sized(twoStations(), 160, 30)

	m = press(m, "ctrl+l")

	if m.focus != logPane {
		t.Fatalf("got %v", m.focus)
	}
}

func Test_given_the_log_focused_on_a_wide_screen_when_ctrl_h_then_the_fleet_takes_focus(t *testing.T) {
	m := press(sized(twoStations(), 160, 30), "ctrl+right")

	m = press(m, "ctrl+h")

	if m.focus != fleetPane {
		t.Fatalf("got %v", m.focus)
	}
}

func Test_given_a_narrow_screen_when_ctrl_l_then_the_fleet_keeps_focus(t *testing.T) {
	m := sized(twoStations(), 80, 24)

	m = press(m, "ctrl+l")

	if m.focus != fleetPane {
		t.Fatalf("got %v", m.focus)
	}
}

func Test_given_a_narrow_screen_when_ctrl_j_then_the_log_takes_focus(t *testing.T) {
	m := sized(twoStations(), 80, 24)

	m = press(m, "ctrl+j")

	if m.focus != logPane {
		t.Fatalf("got %v", m.focus)
	}
}

func Test_given_the_log_focused_on_a_narrow_screen_when_ctrl_up_then_the_fleet_takes_focus(t *testing.T) {
	m := press(sized(twoStations(), 80, 24), "ctrl+j")

	m = press(m, "ctrl+up")

	if m.focus != fleetPane {
		t.Fatalf("got %v", m.focus)
	}
}

func Test_given_the_log_focused_when_k_then_the_log_scrolls_and_the_cursor_stays(t *testing.T) {
	m := press(withLog(sized(twoStations(), 80, 24), 50), "ctrl+j")

	m = press(m, "k", "k")

	if m.logOffset != 2 || m.cursor != 0 {
		t.Fatalf("got offset %d, cursor %d", m.logOffset, m.cursor)
	}
}

func Test_given_the_log_focused_when_gg_then_the_oldest_line_tops_the_pane(t *testing.T) {
	m := press(withLog(sized(twoStations(), 80, 24), 50), "ctrl+j")
	_, logRows := content(m, logPane)

	m = press(m, "g", "g")

	if m.logOffset != 50-logRows || m.cursor != 0 {
		t.Fatalf("got offset %d, cursor %d", m.logOffset, m.cursor)
	}
}

func Test_given_a_scrolled_log_focused_when_G_then_it_follows_the_tail(t *testing.T) {
	m := press(withLog(sized(twoStations(), 80, 24), 50), "ctrl+j", "k", "k")

	m = press(m, "G")

	if m.logOffset != 0 {
		t.Fatalf("got %d", m.logOffset)
	}
}

func Test_given_the_log_focused_when_x_then_no_power_cut_is_sent(t *testing.T) {
	m, sent := recording()
	m = press(sized(m, 80, 24), "ctrl+j")

	press(m, "x")

	if len(*sent) != 0 {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_a_wide_screen_when_plus_then_the_fleet_widens_by_four_columns(t *testing.T) {
	m := sized(twoStations(), 160, 30)
	before := m.fleetSize()

	m = press(m, "+")

	if m.fleetSize() != before+4 {
		t.Fatalf("got %d, want %d", m.fleetSize(), before+4)
	}
}

func Test_given_the_log_focused_on_a_narrow_screen_when_plus_then_the_log_gains_a_row(t *testing.T) {
	m := press(sized(manyStations(10), 80, 24), "ctrl+j")
	_, before := content(m, logPane)

	m = press(m, "+")

	if _, after := content(m, logPane); after != before+1 {
		t.Fatalf("got %d, want %d", after, before+1)
	}
}

func Test_given_a_wide_screen_when_plus_many_times_then_the_log_keeps_its_minimum_width(t *testing.T) {
	m := sized(twoStations(), 160, 30)

	for range 50 {
		m = press(m, "+")
	}

	if log, _ := m.layout().find(logPane); log.width != minPaneColumns {
		t.Fatalf("got %d", log.width)
	}
}

func Test_given_a_resized_split_when_equals_then_the_default_split_returns(t *testing.T) {
	m := sized(twoStations(), 160, 30)
	before := m.fleetSize()
	m = press(m, "+", "+")

	m = press(m, "=")

	if m.fleetSize() != before {
		t.Fatalf("got %d, want %d", m.fleetSize(), before)
	}
}

func Test_given_the_fleet_focused_when_z_then_only_the_fleet_shows(t *testing.T) {
	m := sized(twoStations(), 80, 24)

	m = press(m, "z")

	if text := strings.Join(screen(m, 80, 24), "\n"); strings.Contains(text, "· log") || !strings.Contains(text, "CP2") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_a_zoomed_fleet_when_ctrl_l_on_a_wide_screen_then_only_the_log_shows(t *testing.T) {
	m := press(sized(twoStations(), 160, 30), "z")

	m = press(m, "ctrl+l")

	if text := ansi.Strip(render(m)); !strings.Contains(text, "all stations · log") || strings.Contains(text, "CP2") {
		t.Fatalf("got\n%s", text)
	}
}
