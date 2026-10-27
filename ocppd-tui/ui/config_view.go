package ui

import "github.com/charmbracelet/x/ansi"

const (
	keyWidth   = 40
	valueWidth = 26
)

func (d details) configLines() ([]string, int) {
	c := d.config
	filter := textField(c.filter, "filter keys", 30)
	if c.filtering {
		filter = editingField(c.filter, 30)
	}
	lines := []string{"", "  " + accent.Render("/") + " " + filter, ""}
	shown := d.shown()
	switch {
	case d.described.Station == "":
		lines = append(lines, "  "+dim.Render("waiting for the station…"))
	case len(shown) == 0:
		lines = append(lines, "  "+dim.Render("no key matches"))
	}
	row := min(c.row, max(len(shown)-1, 0))
	for index, s := range shown {
		if index == row {
			lines = append(lines, selectedSetting(s, c.draft))
		} else {
			lines = append(lines, "  "+pad(ansi.Truncate(s.Key, keyWidth-1, "…"), keyWidth)+settingValue(s))
		}
	}
	return append(lines, ""), len(lines) - len(shown) + row
}

func selectedSetting(s setting, draft *string) string {
	line := accent.Render("▌ " + pad(ansi.Truncate(s.Key, keyWidth-1, "…"), keyWidth))
	if draft != nil {
		return line + editingField(*draft, valueWidth)
	}
	return line + settingValue(s)
}

func settingValue(s setting) string {
	switch {
	case s.Readonly && s.Value != nil:
		return dim.Render(pad(" "+ansi.Truncate(*s.Value, valueWidth-2, "…"), valueWidth) + " ro")
	case s.Value == nil:
		return dim.Render(" write-only")
	case s.isBoolean():
		return checkbox(*s.Value == "true", *s.Value)
	}
	return textField(*s.Value, "", valueWidth)
}

func (c configView) footer() string {
	if c.draft != nil || c.filtering {
		return statusLine(insertMode, c.err)
	}
	return statusLine(normalMode, c.err)
}
