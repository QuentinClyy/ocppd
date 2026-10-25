package ui

import "charm.land/lipgloss/v2"

var (
	plain       = lipgloss.NewStyle()
	dim         = lipgloss.NewStyle().Foreground(lipgloss.Color("#8a919c"))
	good        = lipgloss.NewStyle().Foreground(lipgloss.Color("#6fcf97"))
	warn        = lipgloss.NewStyle().Foreground(lipgloss.Color("#f0a35e"))
	accent      = lipgloss.NewStyle().Foreground(lipgloss.Color("#7aa2f7"))
	bold        = lipgloss.NewStyle().Bold(true)
	selected    = lipgloss.NewStyle().Background(lipgloss.Color("#23395b")).Foreground(lipgloss.Color("#d7dae0"))
	cursorBlock = lipgloss.NewStyle().Background(lipgloss.Color("#d7dae0"))
)

func statusStyle(status string) lipgloss.Style {
	switch status {
	case "Charging", "up", "Accepted":
		return good
	case "Faulted", "Unavailable", "down", "Pending", "Rejected":
		return warn
	case "Reserved":
		return accent
	}
	return plain
}
