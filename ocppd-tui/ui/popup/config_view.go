package popup

import (
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
	"github.com/charmbracelet/x/ansi"
)

const (
	keyWidth   = 40
	valueWidth = 26
)

func (p Station) configLines(online bool) ([]string, int) {
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
			lines = append(lines, selectedSetting(s, c.draft, online))
		} else {
			lines = append(lines, "  "+draw.Pad(ansi.Truncate(s.Key, keyWidth-1, "…"), keyWidth)+settingValue(s, online))
		}
	}
	return append(lines, ""), len(lines) - len(shown) + row
}

func selectedSetting(s setting, draft *draw.Editor, online bool) string {
	line := draw.Accent.Render("▌ " + draw.Pad(ansi.Truncate(s.Key, keyWidth-1, "…"), keyWidth))
	if draft != nil {
		return line + draw.EditingField(*draft, valueWidth)
	}
	return line + settingValue(s, online)
}

func settingValue(s setting, online bool) string {
	switch {
	case s.Readonly && online && s.Value != nil:
		return draw.Dim.Render(draw.Pad(" "+ansi.Truncate(*s.Value, valueWidth-2, "…"), valueWidth) + " ro")
	case s.Value == nil:
		return draw.Dim.Render(" write-only")
	case s.isBoolean():
		return draw.Checkbox(*s.Value == "true", *s.Value) + offlineMark(s)
	}
	return draw.TextField(*s.Value, "", valueWidth) + offlineMark(s)
}

func offlineMark(s setting) string {
	if !s.Readonly {
		return ""
	}
	return draw.Dim.Render(" ro")
}

func (c configView) footer() string {
	if c.draft != nil || c.filtering {
		return statusLine(draw.InsertMode, c.err)
	}
	return statusLine(draw.NormalMode, c.err)
}
