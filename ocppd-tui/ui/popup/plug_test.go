package popup

import (
	"testing"

	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/command"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/charmbracelet/x/ansi"
)

func Test_given_an_empty_ev_form_when_enter_then_the_default_ev_is_plugged_in(t *testing.T) {
	f := NewPlug(command.Target{Station: "CP1", Connector: 2})

	line, closed := f.Key(keyMsg("enter"))

	if !closed || line != `{"command":"plug_in","connector":2,"station":"CP1"}` {
		t.Fatalf("got %q, closed %v", line, closed)
	}
}

func Test_given_ev_fields_when_plugged_in_then_they_are_in_the_ev_with_percents_as_fractions(t *testing.T) {
	f := NewPlug(command.Target{Station: "CP1", Connector: 1})
	f.soc, f.target, f.dcKW, f.acPhases = draw.NewEditor("10"), draw.NewEditor("80"), draw.NewEditor("270"), draw.NewEditor("1")

	line, err := f.line()

	want := `{"command":"plug_in","connector":1,"ev":{"ac_phases":1,"dc_kw":270,"soc":0.1,"target_soc":0.8},"station":"CP1"}`
	if err != nil || line != want {
		t.Fatalf("got %q, %v", line, err)
	}
}

func Test_given_a_field_that_is_not_a_number_when_plugged_in_then_an_error_names_the_field(t *testing.T) {
	f := NewPlug(command.Target{Station: "CP1", Connector: 1})
	f.packV = draw.NewEditor("high")

	_, err := f.line()

	if err == nil || err.Error() != "pack V must be a number" {
		t.Fatalf("got %v", err)
	}
}

func Test_given_the_ev_form_when_drawn_then_every_line_has_the_same_width(t *testing.T) {
	f := NewPlug(command.Target{Station: "CP1", Connector: 1})
	f.err = "pack V must be a number"
	frame := f.Frame(40)

	for _, line := range draw.Box(frame.Title, frame.Footer, frame.Lines, plugWidth) {
		if ansi.StringWidth(line) != plugWidth {
			t.Fatalf("got %d: %q", ansi.StringWidth(line), ansi.Strip(line))
		}
	}
}
