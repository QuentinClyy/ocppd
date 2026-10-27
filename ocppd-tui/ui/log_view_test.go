package ui

import (
	"strings"
	"testing"

	"github.com/charmbracelet/x/ansi"
)

func Test_given_a_long_log_following_its_tail_when_rendered_then_the_thumb_is_at_the_bottom_of_the_log_pane(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 200)
	logY, logRows := content(m, logPane)

	lines := strings.Split(ansi.Strip(render(m)), "\n")

	last, first := lines[logY+logRows-1], lines[logY]
	if !strings.HasSuffix(last, "▐") || strings.HasSuffix(first, "▐") {
		t.Fatalf("got first %q, last %q", first, last)
	}
}

func Test_given_a_short_log_when_rendered_then_no_scrollbar(t *testing.T) {
	m := withLog(sized(manyStations(1), 80, 24), 2)

	text := ansi.Strip(render(m))

	if strings.Contains(text, "▐") {
		t.Fatalf("got\n%s", text)
	}
}
