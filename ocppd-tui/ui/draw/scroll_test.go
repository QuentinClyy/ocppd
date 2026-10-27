package draw

import "testing"

func Test_given_lines_that_fit_when_the_thumb_is_placed_then_there_is_none(t *testing.T) {
	start, end := Thumb(5, 10, 0)

	if start != end {
		t.Fatalf("got %d..%d", start, end)
	}
}

func Test_given_a_list_twice_the_pane_when_at_the_top_then_a_half_pane_thumb_at_the_top(t *testing.T) {
	start, end := Thumb(20, 10, 0)

	if start != 0 || end != 5 {
		t.Fatalf("got %d..%d", start, end)
	}
}

func Test_given_a_list_twice_the_pane_when_at_the_end_then_the_thumb_ends_the_pane(t *testing.T) {
	start, end := Thumb(20, 10, 10)

	if start != 5 || end != 10 {
		t.Fatalf("got %d..%d", start, end)
	}
}
