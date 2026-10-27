package ui

type pane int

const (
	fleetPane pane = iota
	logPane
	paneCount
)

const (
	bodyY          = 2
	minPaneRows    = 3
	minPaneColumns = 20
)

type area struct {
	pane   pane
	x, y   int
	width  int
	height int
}

type layout struct {
	side  bool
	areas []area
}

func (m Model) layout() layout {
	l := m.split()
	if !m.zoomed {
		return l
	}
	return layout{side: l.side, areas: []area{{pane: m.focus, y: bodyY, width: m.width, height: max(m.height-4, 0)}}}
}

func (m Model) split() layout {
	body := max(m.height-4, 0)
	if m.width >= wideWidth {
		left := between(m.sideWidth, fleetWidth(wideName)+2, minPaneColumns, m.width-minPaneColumns)
		return layout{side: true, areas: []area{
			{pane: fleetPane, y: bodyY, width: left, height: body},
			{pane: logPane, x: left, y: bodyY, width: m.width - left, height: body},
		}}
	}
	available := max(body-3, 0)
	fallback := min(len(m.visibleRows()), max(available*2/3, 1), available)
	low := min(minPaneRows, available)
	fleetRows := between(m.bottomRows, fallback, low, max(available-minPaneRows, low))
	return layout{areas: []area{
		{pane: fleetPane, y: bodyY, width: m.width, height: 1 + fleetRows},
		{pane: logPane, y: bodyY + 2 + fleetRows, width: m.width, height: 1 + available - fleetRows},
	}}
}

func between(value, fallback, low, high int) int {
	if value == 0 {
		return fallback
	}
	return min(max(value, low), high)
}

func (l layout) find(p pane) (area, bool) {
	for _, a := range l.areas {
		if a.pane == p {
			return a, true
		}
	}
	return area{}, false
}

func (l layout) at(x, y int) (area, bool) {
	for _, a := range l.areas {
		if x >= a.x && x < a.x+a.width && y >= a.y && y < a.y+a.height {
			return a, true
		}
	}
	return area{}, false
}

func (l layout) onDivider(x, y int) bool {
	if len(l.areas) != 2 {
		return false
	}
	first := l.areas[0]
	if l.side {
		return x == first.x+first.width-1 && y >= first.y && y < first.y+first.height
	}
	return y == first.y+first.height
}

func (l layout) neighbour(from pane, direction string) (pane, bool) {
	origin, ok := l.find(from)
	if !ok {
		return from, false
	}
	for _, a := range l.areas {
		if a.pane != from && beyond(origin, a, direction) {
			return a.pane, true
		}
	}
	return from, false
}

func beyond(from, to area, direction string) bool {
	switch direction {
	case "left":
		return to.x+to.width <= from.x
	case "right":
		return to.x >= from.x+from.width
	case "up":
		return to.y+to.height <= from.y
	case "down":
		return to.y >= from.y+from.height
	}
	return false
}

func (m Model) paneRows(p pane) int {
	a, ok := m.layout().find(p)
	if !ok {
		return 0
	}
	return max(a.height-1, 0)
}
