package motion

import "testing"

func feed(keys ...string) (string, int, string) {
	pending, count, motion := "", 0, ""
	for _, key := range keys {
		pending, count, motion = Feed(pending, key, "gy")
	}
	return pending, count, motion
}

func Test_given_no_count_when_a_key_is_fed_then_it_completes_with_count_zero(t *testing.T) {
	pending, count, motion := feed("j")

	if pending != "" || count != 0 || motion != "j" {
		t.Fatalf("got %q %d %q", pending, count, motion)
	}
}

func Test_given_digits_when_a_key_is_fed_then_it_completes_with_their_count(t *testing.T) {
	pending, count, motion := feed("1", "2", "j")

	if pending != "" || count != 12 || motion != "j" {
		t.Fatalf("got %q %d %q", pending, count, motion)
	}
}

func Test_given_digits_when_fed_then_they_stay_pending(t *testing.T) {
	pending, _, motion := feed("1", "0")

	if pending != "10" || motion != "" {
		t.Fatalf("got %q %q", pending, motion)
	}
}

func Test_given_nothing_pending_when_0_is_fed_then_it_is_a_key_not_a_count(t *testing.T) {
	pending, count, motion := feed("0")

	if pending != "" || count != 0 || motion != "0" {
		t.Fatalf("got %q %d %q", pending, count, motion)
	}
}

func Test_given_a_count_when_g_is_fed_then_both_stay_pending(t *testing.T) {
	pending, _, motion := feed("1", "2", "g")

	if pending != "12g" || motion != "" {
		t.Fatalf("got %q %q", pending, motion)
	}
}

func Test_given_a_pending_g_when_g_is_fed_then_gg_completes_with_the_count(t *testing.T) {
	pending, count, motion := feed("1", "2", "g", "g")

	if pending != "" || count != 12 || motion != "gg" {
		t.Fatalf("got %q %d %q", pending, count, motion)
	}
}

func Test_given_a_pending_g_when_another_key_is_fed_then_the_pair_completes(t *testing.T) {
	pending, count, motion := feed("3", "g", "t")

	if pending != "" || count != 3 || motion != "gt" {
		t.Fatalf("got %q %d %q", pending, count, motion)
	}
}

func Test_given_four_digits_when_another_digit_is_fed_then_it_is_ignored(t *testing.T) {
	pending, _, _ := feed("9", "9", "9", "9", "9")

	if pending != "9999" {
		t.Fatalf("got %q", pending)
	}
}

func Test_given_a_count_when_yy_then_it_completes_with_the_count(t *testing.T) {
	pending, count, motion := feed("3", "y", "y")

	if pending != "" || count != 3 || motion != "yy" {
		t.Fatalf("got %q %d %q", pending, count, motion)
	}
}

func Test_given_a_pending_count_when_esc_then_the_count_is_dropped(t *testing.T) {
	pending, count, motion := feed("5", "esc")

	if pending != "" || count != 0 || motion != "" {
		t.Fatalf("got %q %d %q", pending, count, motion)
	}
}

func Test_given_y_is_not_a_prefix_when_y_then_it_completes(t *testing.T) {
	pending, count, motion := Feed("2", "y", "g")

	if pending != "" || count != 2 || motion != "y" {
		t.Fatalf("got %q %d %q", pending, count, motion)
	}
}

func Test_given_a_count_when_j_moves_then_it_stops_on_the_last_index(t *testing.T) {
	index, ok := Move("j", 9, 2, 5)

	if !ok || index != 5 {
		t.Fatalf("got %d %v", index, ok)
	}
}

func Test_given_a_count_when_k_moves_then_it_stops_on_the_first_index(t *testing.T) {
	index, ok := Move("k", 9, 2, 5)

	if !ok || index != 0 {
		t.Fatalf("got %d %v", index, ok)
	}
}

func Test_given_a_count_when_G_moves_then_it_goes_to_that_line(t *testing.T) {
	index, ok := Move("G", 3, 0, 5)

	if !ok || index != 2 {
		t.Fatalf("got %d %v", index, ok)
	}
}

func Test_given_no_count_when_gg_and_G_move_then_first_and_last(t *testing.T) {
	first, _ := Move("gg", 0, 3, 5)
	last, _ := Move("G", 0, 3, 5)

	if first != 0 || last != 5 {
		t.Fatalf("got %d %d", first, last)
	}
}

func Test_given_another_key_when_moved_then_the_index_stays(t *testing.T) {
	index, ok := Move("p", 0, 3, 5)

	if ok || index != 3 {
		t.Fatalf("got %d %v", index, ok)
	}
}
