package command

import "testing"

func Test_given_each_connector_key_when_built_then_its_command_for_that_connector(t *testing.T) {
	at := Target{Station: "CP1", Connector: 2}
	want := map[string]string{
		"p": `{"command":"plug_in","connector":2,"station":"CP1"}`,
		"s": `{"command":"stop","connector":2,"station":"CP1"}`,
		"u": `{"command":"unplug","connector":2,"station":"CP1"}`,
		"f": `{"command":"fault","connector":2,"station":"CP1"}`,
		"c": `{"command":"clear_fault","connector":2,"station":"CP1"}`,
	}

	for key, line := range want {
		got, ok := Driver(key, at)

		if !ok || got != line {
			t.Fatalf("%s: got %q", key, got)
		}
	}
}

func Test_given_x_on_a_station_when_built_then_a_power_cut_for_the_station(t *testing.T) {
	at := Target{Station: "CP1"}

	line, ok := Driver("x", at)

	if !ok || line != `{"command":"power_cut","station":"CP1"}` {
		t.Fatalf("got %q", line)
	}
}
