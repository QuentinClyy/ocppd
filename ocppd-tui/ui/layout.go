package ui

type layout struct {
	wide      bool
	fleetY    int
	fleetRows int
	logY      int
	logRows   int
	split     int
}

func (m Model) layout() layout {
	if m.width >= wideWidth {
		rows := max(m.height-5, 0)
		return layout{wide: true, fleetY: 3, fleetRows: rows, logY: 3, logRows: rows, split: 1 + fleetWidth(wideName)}
	}
	available := max(m.height-7, 0)
	fleetRows := min(len(rows(m.fleet, m.expanded)), max(available*2/3, 1), available)
	return layout{fleetY: 3, fleetRows: fleetRows, logY: 5 + fleetRows, logRows: available - fleetRows}
}
