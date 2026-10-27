package ui

import "github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"

const minModalWidth = 16

func (m Model) modal(f draw.Frame) []string {
	rows := m.modalRows()
	top := min(max(f.Top, 0), max(len(f.Lines)-rows, 0))
	framed := draw.Box(f.Title, f.Footer, draw.Window(f.Lines, top, rows), max(min(f.Width, m.width-2), minModalWidth))
	return draw.Centered(framed, m.width, m.height-4)
}

func (m Model) modalRows() int {
	return max(m.height-6, 1)
}
