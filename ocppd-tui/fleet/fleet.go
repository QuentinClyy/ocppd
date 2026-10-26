package fleet

import "slices"

type Connector struct {
	ID          int
	Status      string
	Transaction string
	PowerW      float64
	Soc         float64
	Sampled     bool
}

type Station struct {
	ID         string
	Ocpp       string
	Link       string
	Boot       string
	Connectors []Connector
}

type Fleet struct {
	Stations []Station
}

func (f *Fleet) Apply(raw string) (string, bool) {
	e, ok := parseEvent(raw)
	if !ok {
		return "", false
	}
	f.apply(e)
	return e.Station, true
}

func (f *Fleet) Remove(id string) {
	f.Stations = slices.DeleteFunc(f.Stations, func(s Station) bool { return s.ID == id })
}

var tracked = map[string]bool{
	"connected":           true,
	"disconnected":        true,
	"connect_failed":      true,
	"boot_status":         true,
	"status_changed":      true,
	"transaction_started": true,
	"transaction_event":   true,
	"transaction_stopped": true,
	"meter_sample":        true,
}

func (f *Fleet) apply(e event) {
	if !tracked[e.Kind] {
		return
	}
	s := f.station(e.Station)
	switch e.Kind {
	case "connected":
		s.Link = "up"
	case "disconnected", "connect_failed":
		s.Link = "down"
	case "boot_status":
		s.Boot = e.Status
	case "status_changed":
		if e.slot() == 0 {
			return
		}
		s.Ocpp = version(e)
		s.connector(e.slot()).Status = e.Status
	case "transaction_started":
		s.connector(e.slot()).Transaction = e.transaction()
	case "transaction_event":
		c := s.connector(e.slot())
		c.Transaction = e.transaction()
		if e.EventType == "Ended" {
			c.idle()
		}
	case "transaction_stopped":
		s.connector(e.slot()).idle()
	case "meter_sample":
		c := s.connector(e.slot())
		c.PowerW, c.Soc, c.Sampled = e.PowerW, e.Soc, true
	}
}

func (f *Fleet) station(id string) *Station {
	index, found := slices.BinarySearchFunc(f.Stations, id, func(s Station, id string) int { return naturalCompare(s.ID, id) })
	if !found {
		f.Stations = slices.Insert(f.Stations, index, Station{ID: id})
	}
	return &f.Stations[index]
}

func (s *Station) connector(id int) *Connector {
	index, found := slices.BinarySearchFunc(s.Connectors, id, func(c Connector, id int) int { return c.ID - id })
	if !found {
		s.Connectors = slices.Insert(s.Connectors, index, Connector{ID: id})
	}
	return &s.Connectors[index]
}

func (c *Connector) idle() {
	c.Transaction, c.PowerW, c.Sampled = "", 0, false
}

func version(e event) string {
	if e.Evse > 0 {
		return "2.0.1"
	}
	return "1.6"
}
