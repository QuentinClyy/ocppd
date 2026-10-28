package draw

import "charm.land/lipgloss/v2"

var (
	Plain       = lipgloss.NewStyle()
	Dim         = lipgloss.NewStyle().Foreground(lipgloss.Color("#8a919c"))
	Good        = lipgloss.NewStyle().Foreground(lipgloss.Color("#6fcf97"))
	Warn        = lipgloss.NewStyle().Foreground(lipgloss.Color("#f0a35e"))
	Accent      = lipgloss.NewStyle().Foreground(lipgloss.Color("#7aa2f7"))
	Bold        = lipgloss.NewStyle().Bold(true)
	Selected    = lipgloss.NewStyle().Background(lipgloss.Color("#23395b")).Foreground(lipgloss.Color("#d7dae0"))
	Idle        = lipgloss.NewStyle().Background(lipgloss.Color("#3a3f4b")).Foreground(lipgloss.Color("#d7dae0"))
	Heading     = lipgloss.NewStyle().Foreground(lipgloss.Color("#8a919c")).Bold(true)
	Field       = lipgloss.NewStyle().Background(lipgloss.Color("#262a38"))
	FieldHint   = Field.Foreground(lipgloss.Color("#8a919c"))
	Chosen      = lipgloss.NewStyle().Background(lipgloss.Color("#7aa2f7")).Foreground(lipgloss.Color("#0b0d10")).Bold(true)
	insertBadge = Chosen
	normalBadge = lipgloss.NewStyle().Background(lipgloss.Color("#6fcf97")).Foreground(lipgloss.Color("#0b0d10")).Bold(true)
	ActiveTab   = Accent.Bold(true).Underline(true)
	CursorBlock = lipgloss.NewStyle().Background(lipgloss.Color("#d7dae0")).Foreground(lipgloss.Color("#0b0d10"))
)

var (
	InsertMode = insertBadge.Render(" INSERT ")
	NormalMode = normalBadge.Render(" NORMAL ")
)

func StatusStyle(status string) lipgloss.Style {
	switch status {
	case "Charging", "up", "Accepted":
		return Good
	case "Faulted", "Unavailable", "down", "Pending", "Rejected":
		return Warn
	case "Reserved":
		return Accent
	case "off":
		return Dim
	}
	return Plain
}
