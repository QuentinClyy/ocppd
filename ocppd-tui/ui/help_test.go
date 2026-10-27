package ui

import (
	"strings"
	"testing"

	"github.com/charmbracelet/x/ansi"
)

func Test_given_help_when_opened_then_the_stations_and_global_bindings_are_listed(t *testing.T) {
	m, _ := recording()

	m = press(m, "?")

	text := ansi.Strip(strings.Join(screen(m, 80, 30), "\n"))
	for _, b := range bindingsFor(fleetPane) {
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
		bar := ansi.Strip(keyBar(width, fleetPane))

		if ansi.StringWidth(bar) > width || !strings.HasSuffix(bar, "? help") {
			t.Fatalf("%d: got %q", width, bar)
		}
	}
}

func Test_given_the_help_box_when_drawn_then_every_line_has_the_same_width(t *testing.T) {
	lines := helpLines(fleetPane)

	for _, line := range lines {
		if ansi.StringWidth(line) != helpWidth {
			t.Fatalf("got %d: %q", ansi.StringWidth(line), ansi.Strip(line))
		}
	}
}

func Test_given_the_log_focused_when_help_opens_then_driver_keys_are_not_listed(t *testing.T) {
	m := press(sized(twoStations(), 80, 30), "ctrl+j")

	m = press(m, "?")

	text := strings.Join(screen(m, 80, 30), "\n")
	if strings.Contains(text, "plug in an EV") || !strings.Contains(text, "oldest / newest line") {
		t.Fatalf("got\n%s", text)
	}
}

func Test_given_the_log_focused_when_the_key_bar_is_built_then_it_has_no_driver_keys(t *testing.T) {
	bar := ansi.Strip(keyBar(160, logPane))

	if strings.Contains(bar, "plug") || !strings.Contains(bar, "j/k scroll") {
		t.Fatalf("got %q", bar)
	}
}
