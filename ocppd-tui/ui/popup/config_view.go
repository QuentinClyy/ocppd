package popup

import (
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/charmbracelet/x/ansi"
)

const (
	keyWidth   = 40
	valueWidth = 26
)

func (p Station) configLines() ([]string, int) {
	c := p.config
	filter := draw.TextField(c.filter.String(), "filter keys", 30)
	if c.filtering {
		filter = draw.EditingField(c.filter, 30)
	}
	lines := []string{"", "  " + draw.Accent.Render("/") + " " + filter, ""}
	shown := p.shown()
	switch {
	case p.described.Station == "":
		lines = append(lines, "  "+draw.Dim.Render("waiting for the station…"))
	case len(shown) == 0:
		lines = append(lines, "  "+draw.Dim.Render("no key matches"))
	}
	row := min(c.row, max(len(shown)-1, 0))
	for index, s := range shown {
		if index == row {
			lines = append(lines, selectedSetting(s, c.draft))
		} else {
			lines = append(lines, "  "+draw.Pad(ansi.Truncate(s.Key, keyWidth-1, "…"), keyWidth)+settingValue(s))
		}
	}
	return append(lines, ""), len(lines) - len(shown) + row
}

func selectedSetting(s setting, draft *draw.Editor) string {
	line := draw.Accent.Render("▌ " + draw.Pad(ansi.Truncate(s.Key, keyWidth-1, "…"), keyWidth))
	if draft != nil {
		return line + draw.EditingField(*draft, valueWidth)
	}
	return line + settingValue(s)
}

func settingValue(s setting) string {
	switch {
	case s.Readonly && s.Value != nil:
		return draw.Dim.Render(draw.Pad(" "+ansi.Truncate(*s.Value, valueWidth-2, "…"), valueWidth) + " ro")
	case s.Value == nil:
		return draw.Dim.Render(" write-only")
	case s.isBoolean():
		return draw.Checkbox(*s.Value == "true", *s.Value)
	}
	return draw.TextField(*s.Value, "", valueWidth)
}

func (c configView) footer() string {
	if c.draft != nil || c.filtering {
		return statusLine(draw.InsertMode, c.err)
	}
	return statusLine(draw.NormalMode, c.err)
}
