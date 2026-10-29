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

func (m Model) press(msg tea.MouseClickMsg) Model {
	if msg.Button != tea.MouseLeft {
		return m
	}
	m.yanked = 0
	at := point{msg.X, msg.Y}
	l := m.layout()
	switch a, ok := l.at(msg.X, msg.Y); {
	case m.overlaid():
		m.sel = selection{bounds: area{width: m.width, height: m.height}, from: at, to: at, live: true}
	case l.onDivider(msg.X, msg.Y):
		m.dragging = true
	case ok:
		a.width--
		m.sel = selection{bounds: a, from: at, to: at, live: true}
	}
	return m
}

func (m Model) motion(msg tea.MouseMotionMsg) Model {
	switch {
	case m.dragging:
		return m.dragTo(msg.X, msg.Y)
	case m.sel.live:
		m.sel = m.sel.extend(msg.X, msg.Y)
	}
	return m
}

func (m Model) lift() (Model, tea.Cmd) {
	sel := m.sel
	m.dragging, m.sel = false, selection{}
	switch {
	case !sel.live || (!sel.moved() && m.overlaid()):
		return m, nil
	case !sel.moved():
		m = m.click(sel.from)
		m.fleetScroll = m.fleetTop()
		return m, nil
	}
	start, end := sel.span()
	text := selectedText(frame(m), sel)
	if text == "" {
		return m, nil
	}
	m.yanked = end.y - start.y + 1
	return m, tea.SetClipboard(text)
}

func (m Model) click(at point) Model {
	l := m.layout()
	a, ok := l.at(at.x, at.y)
	if !ok {
		return m
	}
	m.focus = a.pane
	if a.pane == fleetPane {
		return m.clickRow(at.y - a.y - 1)
	}
	return m.clickLog(at.y - a.y - 1)
}

func (m Model) clickLog(index int) Model {
	total := len(m.logLines())
	clicked := max(total-m.paneRows(logPane)-m.clampLog(m.logOffset), 0) + index
	if index < 0 || clicked >= total {
		return m
	}
	return m.moveLog(total - 1 - clicked)
}

func (m Model) clickRow(index int) Model {
	total := len(m.visibleRows())
	clicked := m.fleetTop() + index
	switch {
	case index < 0 || clicked >= total:
		return m
	case clicked == min(m.cursor, total-1):
		return m.move("enter", 0)
	}
	m.cursor = clicked
	return m
}
