package ui

import (
	"strings"
	"testing"

	"github.com/charmbracelet/x/ansi"
)

func Test_given_help_when_opened_then_every_binding_is_listed(t *testing.T) {
	m, _ := recording()

	m = press(m, "?")

	text := ansi.Strip(strings.Join(screen(m, 80, 30), "\n"))
	for _, b := range bindings {
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
		bar := ansi.Strip(keyBar(width))

		if ansi.StringWidth(bar) > width || !strings.HasSuffix(bar, "? help") {
			t.Fatalf("%d: got %q", width, bar)
		}
	}
}

func Test_given_the_help_box_when_drawn_then_every_line_has_the_same_width(t *testing.T) {
	lines := helpLines()

	for _, line := range lines {
		if ansi.StringWidth(line) != helpWidth {
			t.Fatalf("got %d: %q", ansi.StringWidth(line), ansi.Strip(line))
		}
	}
}
