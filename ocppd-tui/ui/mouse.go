package ui

import tea "charm.land/bubbletea/v2"

func (m Model) wheel(msg tea.MouseWheelMsg) Model {
	var older int
	switch msg.Button {
	case tea.MouseWheelUp:
		older = wheelStep
	case tea.MouseWheelDown:
		older = -wheelStep
	default:
		return m
	}
	a, ok := m.layout().at(msg.X, msg.Y)
	switch {
	case !ok:
		return m
	case a.pane == logPane:
		return m.scrollLog(older)
	}
	return m.scrollFleet(-older)
}

func (m Model) click(msg tea.MouseClickMsg) Model {
	if msg.Button != tea.MouseLeft {
		return m
	}
	l := m.layout()
	if l.onDivider(msg.X, msg.Y) {
		m.dragging = true
		return m
	}
	a, ok := l.at(msg.X, msg.Y)
	if !ok {
		return m
	}
	m.focus = a.pane
	if a.pane == fleetPane {
		return m.clickRow(msg.Y - a.y - 1)
	}
	return m
}

func (m Model) clickRow(index int) Model {
	total := len(m.visibleRows())
	clicked := m.fleetTop() + index
	switch {
	case index < 0 || clicked >= total:
		return m
	case clicked == min(m.cursor, total-1):
		return m.move("enter")
	}
	m.cursor = clicked
	return m
}
