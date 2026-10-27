package ui

import (
	"testing"

	tea "charm.land/bubbletea/v2"
)

func sized(m Model, width, height int) Model {
	m.width, m.height = width, height
	return m
}

func content(m Model, p pane) (int, int) {
	a, _ := m.layout().find(p)
	return a.y + 1, a.height - 1
}

func wheelAt(m Model, button tea.MouseButton, x, y, times int) Model {
	for range times {
		next, _ := m.Update(tea.MouseWheelMsg{Button: button, X: x, Y: y})
		m = next.(Model)
	}
	return m
}

func Test_given_a_wide_screen_when_the_wheel_scrolls_left_of_the_divider_then_the_fleet_scrolls(t *testing.T) {
	m := withLog(sized(manyStations(40), 160, 30), 50)

	m = wheelAt(m, tea.MouseWheelDown, 10, 10, 1)

	if m.fleetTop() != 3 || m.logOffset != 0 {
		t.Fatalf("got top %d, offset %d", m.fleetTop(), m.logOffset)
	}
}

func Test_given_a_wide_screen_when_the_wheel_scrolls_right_of_the_divider_then_the_log_scrolls(t *testing.T) {
	m := withLog(sized(manyStations(40), 160, 30), 50)

	m = wheelAt(m, tea.MouseWheelUp, 120, 10, 1)

	if m.fleetTop() != 0 || m.logOffset != 3 {
		t.Fatalf("got top %d, offset %d", m.fleetTop(), m.logOffset)
	}
}
