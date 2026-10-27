package ui

var focusKeys = map[string]string{
	"ctrl+h": "left", "ctrl+left": "left",
	"ctrl+j": "down", "ctrl+down": "down",
	"ctrl+k": "up", "ctrl+up": "up",
	"ctrl+l": "right", "ctrl+right": "right",
}

func (m Model) focusToward(direction string) Model {
	if next, ok := m.split().neighbour(m.focus, direction); ok {
		m.focus = next
	}
	return m
}

func (m Model) fleetSize() int {
	fleet, _ := m.split().find(fleetPane)
	if m.split().side {
		return fleet.width
	}
	return fleet.height - 1
}

func (m Model) resizeFleet(size int) Model {
	if m.split().side {
		m.sideWidth = max(size, 1)
		m.sideWidth = m.fleetSize()
		return m
	}
	m.bottomRows = max(size, 1)
	m.bottomRows = m.fleetSize()
	return m
}

func (m Model) grow(steps int) Model {
	step := 1
	if m.split().side {
		step = 4
	}
	if m.focus != fleetPane {
		step = -step
	}
	return m.resizeFleet(m.fleetSize() + steps*step)
}

func (m Model) dragTo(x, y int) Model {
	if m.split().side {
		return m.resizeFleet(x + 1)
	}
	return m.resizeFleet(y - bodyY - 1)
}
