package ui

import (
	"strings"
	"testing"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ocppd"
	"github.com/charmbracelet/x/ansi"
)

func charging() Model {
	m := New(func(string) error { return nil })
	for _, line := range []string{
		`{"station":"EVB-0001","at":1,"kind":"connected"}`,
		`{"station":"EVB-0001","at":2,"kind":"boot_status","status":"Accepted"}`,
		`{"station":"EVB-0001","at":3,"kind":"status_changed","connector":1,"status":"Charging"}`,
		`{"station":"EVB-0001","at":3,"kind":"status_changed","connector":2,"status":"Available"}`,
		`{"station":"EVB-0001","at":4,"kind":"transaction_started","connector":1,"transaction_id":1042}`,
		`{"station":"EVB-0001","at":5,"kind":"meter_sample","connector":1,"register_wh":9.0,"power_w":11000.0,"soc":0.47}`,
	} {
		next, _ := m.Update(ocppd.EventMsg(line))
		m = next.(Model)
	}
	return expandAll(m)
}

func screen(m Model, width, height int) []string {
	m.width, m.height = width, height
	return strings.Split(ansi.Strip(render(m)), "\n")
}

func Test_given_a_charging_station_when_rendered_narrow_then_the_station_and_its_connectors_as_a_tree(t *testing.T) {
	m := charging()

	lines := screen(m, 80, 24)

	want := []string{
		"   STATION               OCPP   LINK  BOOT      STATUS       kW     SOC   TX",
		" ▾ EVB-0001              1.6    up    Accepted  1 charging   11.0",
		"   ├─ 1                                         Charging     11.0   47%   1042",
		"   └─ 2                                         Available    —      —",
	}
	for index, line := range want {
		if strings.TrimRight(lines[2+index], " ") != line {
			t.Fatalf("line %d\ngot  %q\nwant %q", 2+index, lines[2+index], line)
		}
	}
}

func Test_given_any_size_when_rendered_then_the_screen_is_exactly_the_terminal_height(t *testing.T) {
	m := charging()

	for _, size := range [][2]int{{80, 24}, {100, 40}, {160, 30}} {
		lines := screen(m, size[0], size[1])

		if len(lines) != size[1] {
			t.Fatalf("%v: got %d lines", size, len(lines))
		}
	}
}

func Test_given_a_wide_terminal_when_rendered_then_the_log_sits_beside_the_fleet(t *testing.T) {
	m := charging()

	lines := screen(m, 160, 30)

	if !strings.Contains(lines[2], "STATION") || !strings.Contains(lines[2], "│ LOGS · all stations") {
		t.Fatalf("got %q", lines[2])
	}
}

func Test_given_the_log_filtered_when_a_station_is_selected_then_the_log_title_names_it(t *testing.T) {
	m := charging()

	m = press(m, "tab")

	if !strings.Contains(strings.Join(screen(m, 80, 24), "\n"), "LOGS · EVB-0001") {
		t.Fatal("no station log title")
	}
}

func Test_given_a_line_wider_than_the_terminal_when_rendered_then_it_is_cut_to_the_width(t *testing.T) {
	m := charging()

	for _, line := range screen(m, 40, 24) {
		if ansi.StringWidth(line) > 40 {
			t.Fatalf("got %d columns: %q", ansi.StringWidth(line), line)
		}
	}
}

func Test_given_q_when_pressed_then_the_program_quits(t *testing.T) {
	m := New(func(string) error { return nil })

	_, cmd := m.Update(tea.KeyPressMsg{Code: 'q', Text: "q"})

	if cmd == nil {
		t.Fatal("no command")
	}
	if _, quit := cmd().(tea.QuitMsg); !quit {
		t.Fatalf("got %T", cmd())
	}
}

func Test_given_one_station_when_rendered_then_the_header_says_1_station(t *testing.T) {
	m := charging()

	header := screen(m, 80, 24)[0]

	if !strings.HasSuffix(strings.TrimRight(header, " "), "1 station") {
		t.Fatalf("got %q", header)
	}
}

func Test_given_a_csms_url_longer_than_the_header_when_rendered_then_it_is_cut_before_the_station_count(t *testing.T) {
	m := charging()
	m.csms = "ws://host.docker.internal:8180/steve/websocket/CentralSystemService"

	header := screen(m, 60, 24)[0]

	if ansi.StringWidth(header) > 60 || !strings.HasSuffix(header, "… 1 station") {
		t.Fatalf("got %q", header)
	}
}

func greeted(expected, daemon string) Model {
	m := charging().Expecting(expected)
	next, _ := m.Update(ocppd.EventMsg(`{"kind":"daemon","csms":"ws://x","version":"` + daemon + `"}`))
	return next.(Model)
}

func Test_given_an_older_daemon_when_rendered_then_the_header_warns_and_says_how_to_restart(t *testing.T) {
	m := greeted("0.3.0", "0.2.0")

	header := screen(m, 100, 24)[0]

	if !strings.Contains(header, "daemon 0.2.0 ≠ 0.3.0 · Q restarts") || !strings.HasSuffix(strings.TrimRight(header, " "), "1 station") {
		t.Fatalf("got %q", header)
	}
}

func Test_given_a_daemon_of_the_same_version_when_rendered_then_no_warning(t *testing.T) {
	m := greeted("0.3.0", "0.3.0")

	header := screen(m, 100, 24)[0]

	if strings.Contains(header, "≠") {
		t.Fatalf("got %q", header)
	}
}

func Test_given_a_tui_started_without_an_expected_version_when_rendered_then_no_warning(t *testing.T) {
	m := greeted("", "0.2.0")

	header := screen(m, 100, 24)[0]

	if strings.Contains(header, "≠") {
		t.Fatalf("got %q", header)
	}
}

func Test_given_a_shutdown_asked_for_when_the_connection_closes_then_it_quits_without_reporting_a_loss(t *testing.T) {
	m, _ := recording()
	m = press(m, "Q", "y")

	next, cmd := m.Update(ocppd.ClosedMsg{})

	if next.(Model).Lost() || cmd == nil {
		t.Fatalf("lost %v, cmd %v", next.(Model).Lost(), cmd)
	}
}

func Test_given_no_shutdown_when_the_connection_closes_then_it_quits_and_reports_the_loss(t *testing.T) {
	m, _ := recording()

	next, cmd := m.Update(ocppd.ClosedMsg{})

	if !next.(Model).Lost() || cmd == nil {
		t.Fatalf("lost %v, cmd %v", next.(Model).Lost(), cmd)
	}
}

func Test_given_the_daemon_line_when_received_then_the_header_shows_its_url_and_the_log_skips_it(t *testing.T) {
	m, _ := recording()

	next, _ := m.Update(ocppd.EventMsg(`{"kind":"daemon","csms":"ws://steve/ocpp","version":"0.2.0"}`))

	m = next.(Model)
	if m.csms != "ws://steve/ocpp" || len(m.log) != 0 {
		t.Fatalf("csms %q, log %v", m.csms, m.log)
	}
}
