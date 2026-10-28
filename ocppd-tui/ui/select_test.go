package ui

import (
	"strings"
	"testing"

	"github.com/charmbracelet/x/ansi"
)

var sampleLines = []string{"zero   | left", "one    | right  ", "two    | more"}

func within(x, y, width, height int) selection {
	return selection{bounds: area{x: x, y: y, width: width, height: height}}
}

func Test_given_a_drag_across_lines_when_text_is_picked_then_first_and_last_lines_are_cut_and_trailing_spaces_dropped(t *testing.T) {
	s := within(9, 0, 8, 3)
	s.from, s.to = point{11, 0}, point{12, 2}

	got := selectedText(sampleLines, s)

	if want := "ft\nright\nmore"; got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}

func Test_given_a_drag_that_goes_backwards_when_text_is_picked_then_it_reads_forwards(t *testing.T) {
	s := within(0, 0, 6, 3)
	s.from, s.to = point{2, 1}, point{0, 0}

	if got := selectedText(sampleLines, s); got != "zero\none" {
		t.Fatalf("got %q", got)
	}
}

func Test_given_styled_wide_text_when_picked_then_columns_count_cells_and_styles_are_dropped(t *testing.T) {
	s := within(0, 0, 10, 1)
	s.from, s.to = point{1, 0}, point{3, 0}

	if got := selectedText([]string{"\x1b[31mあい\x1b[0mxyz"}, s); got != "あい" {
		t.Fatalf("got %q", got)
	}
}

func Test_given_a_live_selection_when_highlighted_then_the_text_is_unchanged_and_only_the_span_is_styled(t *testing.T) {
	s := within(0, 0, 14, 3)
	s.from, s.to, s.live = point{2, 1}, point{4, 1}, true

	shown := highlight(sampleLines, s)

	if ansi.Strip(strings.Join(shown, "\n")) != strings.Join(sampleLines, "\n") || shown[0] != sampleLines[0] || shown[1] == sampleLines[1] {
		t.Fatalf("got %q", shown)
	}
}

func Test_given_the_log_when_dragged_across_lines_then_the_text_is_copied_and_the_rows_are_untouched(t *testing.T) {
	m := withLog(sized(collapsed(), 80, 24), 10)
	_, logY := content(m, logPane)
	before := m.logCursor

	m = pressAt(m, 5, logY)
	m = dragTo(m, 30, logY+1)
	m, cmd := releaseCmd(m)

	if cmd == nil || m.yanked != 2 || m.logCursor != before || m.sel.live {
		t.Fatalf("cmd %v, yanked %d, cursor %d, live %v", cmd, m.yanked, m.logCursor, m.sel.live)
	}
}

func Test_given_the_selected_station_when_a_drag_starts_on_it_then_it_is_not_toggled_open(t *testing.T) {
	m := collapsed()
	fleetY, _ := content(m, fleetPane)

	m = pressAt(m, 10, fleetY)
	m = dragTo(m, 40, fleetY)
	m = release(m)

	if m.expanded["CP1"] {
		t.Fatal("drag opened the station")
	}
}

func Test_given_a_drag_that_leaves_the_pane_when_picked_then_it_stays_inside_the_pane(t *testing.T) {
	m := sized(collapsed(), 160, 30)
	fleetY, _ := content(m, fleetPane)

	m = pressAt(m, 10, fleetY)
	m = dragTo(m, 150, fleetY)

	if m.sel.to.x >= m.fleetSize() {
		t.Fatalf("selection reached x=%d past the fleet pane of %d", m.sel.to.x, m.fleetSize())
	}
}

func Test_given_the_spawn_form_when_dragged_across_it_then_the_text_is_copied(t *testing.T) {
	m := sized(collapsed(), 80, 24)
	m = press(m, "n")

	m = pressAt(m, 2, 5)
	m = dragTo(m, 30, 8)
	m, cmd := releaseCmd(m)

	if cmd == nil || m.form == nil {
		t.Fatalf("cmd %v, form %v", cmd, m.form)
	}
}
