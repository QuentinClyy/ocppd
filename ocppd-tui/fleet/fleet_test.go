package fleet

import (
	"strings"
	"testing"
)

func applied(lines ...string) Fleet {
	var f Fleet
	for _, line := range lines {
		f.Apply(line)
	}
	return f
}

func Test_given_a_booted_1_6_station_when_applied_then_up_accepted_with_its_connectors(t *testing.T) {
	f := applied(
		`{"station":"CP1","at":1,"kind":"connected"}`,
		`{"station":"CP1","at":2,"kind":"boot_status","status":"Accepted"}`,
		`{"station":"CP1","at":3,"kind":"status_changed","connector":0,"status":"Available"}`,
		`{"station":"CP1","at":3,"kind":"status_changed","connector":2,"status":"Available"}`,
		`{"station":"CP1","at":3,"kind":"status_changed","connector":1,"status":"Available"}`,
	)

	s := f.Stations[0]

	if s.ID != "CP1" || s.Ocpp != "1.6" || s.Link != "up" || s.Boot != "Accepted" {
		t.Fatalf("got %+v", s)
	}
	if len(s.Connectors) != 2 || s.Connectors[0].ID != 1 || s.Connectors[1].ID != 2 {
		t.Fatalf("got %+v", s.Connectors)
	}
}

func Test_given_a_1_6_transaction_with_a_sample_when_applied_then_the_connector_shows_it(t *testing.T) {
	f := applied(
		`{"station":"CP1","at":1,"kind":"status_changed","connector":1,"status":"Charging"}`,
		`{"station":"CP1","at":2,"kind":"transaction_started","connector":1,"transaction_id":1042}`,
		`{"station":"CP1","at":3,"kind":"meter_sample","connector":1,"register_wh":120.0,"power_w":11000.0,"soc":0.47}`,
	)

	c := f.Stations[0].Connectors[0]

	if c.Status != "Charging" || c.Transaction != "1042" || c.PowerW != 11000 || c.Soc != 0.47 || !c.Sampled {
		t.Fatalf("got %+v", c)
	}
}

func Test_given_a_1_6_transaction_when_stopped_then_the_connector_is_idle(t *testing.T) {
	f := applied(
		`{"station":"CP1","at":2,"kind":"transaction_started","connector":1,"transaction_id":1042}`,
		`{"station":"CP1","at":3,"kind":"meter_sample","connector":1,"register_wh":120.0,"power_w":11000.0,"soc":0.47}`,
		`{"station":"CP1","at":4,"kind":"transaction_stopped","connector":1,"transaction_id":1042,"reason":"Local"}`,
	)

	c := f.Stations[0].Connectors[0]

	if c.Transaction != "" || c.Sampled || c.PowerW != 0 {
		t.Fatalf("got %+v", c)
	}
}

func Test_given_a_2_0_1_transaction_with_a_sample_when_applied_then_the_evse_shows_it(t *testing.T) {
	f := applied(
		`{"station":"CS1","at":1,"kind":"status_changed","evse":1,"connector":1,"status":"Occupied"}`,
		`{"station":"CS1","at":2,"kind":"transaction_event","evse":1,"transaction_id":"1a17539c66b-1","event_type":"Started","trigger_reason":"CablePluggedIn"}`,
		`{"station":"CS1","at":3,"kind":"meter_sample","evse":1,"register_wh":0.0,"power_w":22000.0,"soc":0.31}`,
	)

	s := f.Stations[0]

	if s.Ocpp != "2.0.1" || len(s.Connectors) != 1 {
		t.Fatalf("got %+v", s)
	}
	if c := s.Connectors[0]; c.Status != "Occupied" || c.Transaction != "1a17539c66b-1" || c.PowerW != 22000 {
		t.Fatalf("got %+v", c)
	}
}

func Test_given_a_2_0_1_transaction_when_ended_then_the_evse_is_idle(t *testing.T) {
	f := applied(
		`{"station":"CS1","at":2,"kind":"transaction_event","evse":1,"transaction_id":"t1","event_type":"Started","trigger_reason":"CablePluggedIn"}`,
		`{"station":"CS1","at":3,"kind":"meter_sample","evse":1,"register_wh":0.0,"power_w":22000.0,"soc":0.31}`,
		`{"station":"CS1","at":4,"kind":"transaction_event","evse":1,"transaction_id":"t1","event_type":"Ended","trigger_reason":"EVCommunicationLost"}`,
	)

	c := f.Stations[0].Connectors[0]

	if c.Transaction != "" || c.Sampled {
		t.Fatalf("got %+v", c)
	}
}

func Test_given_stations_seen_out_of_order_when_applied_then_listed_in_natural_id_order(t *testing.T) {
	f := applied(
		`{"station":"CP10","at":1,"kind":"connected"}`,
		`{"station":"CP2","at":2,"kind":"connected"}`,
		`{"station":"CP1","at":3,"kind":"connected"}`,
		`{"station":"CP10","at":4,"kind":"disconnected"}`,
	)

	var ids []string
	for _, s := range f.Stations {
		ids = append(ids, s.ID)
	}

	if strings.Join(ids, ",") != "CP1,CP2,CP10" || f.Stations[2].Link != "down" {
		t.Fatalf("got %v %+v", ids, f.Stations)
	}
}

func Test_given_ids_with_and_without_numbers_when_compared_then_numbers_sort_by_value(t *testing.T) {
	pairs := [][2]string{{"CP2", "CP10"}, {"A", "B"}, {"EVB-0009", "EVB-0010"}, {"CS", "CS1"}}

	for _, pair := range pairs {
		order := naturalCompare(pair[0], pair[1])

		if order >= 0 || naturalCompare(pair[1], pair[0]) <= 0 {
			t.Fatalf("%v: got %d", pair, order)
		}
	}
}

func Test_given_a_rejected_spawn_naming_a_station_when_applied_then_no_station_appears(t *testing.T) {
	f := applied(`{"station":"CP1","at":1,"kind":"command_rejected","command":"spawn","reason":"phases must be 1 to 3"}`)

	stations := f.Stations

	if len(stations) != 0 {
		t.Fatalf("got %+v", stations)
	}
}
