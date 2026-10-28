package popup

import (
	"fmt"
	"strings"
	"time"

	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"
)

const stationWidth = 78

type Context struct {
	Station fleet.Station
	Recent  []string
	Zone    *time.Location
}

func (p Station) Frame(c Context, rows int) draw.Frame {
	f := draw.Frame{Title: tabbed(p.id, p.tab), Width: stationWidth}
	if p.tab == detailsTab {
		f.Lines = p.detailsLines(c)
		return f
	}
	lines, focused := p.configLines(c.Station.Link == "up")
	f.Lines, f.Top, f.Footer = lines, focused-rows+1, p.config.footer()
	return f
}

func (p Station) detailsLines(c Context) []string {
	s, d := c.Station, p.described
	since := draw.Dim.Render("—")
	if s.Since != 0 {
		since = time.UnixMilli(s.Since).In(c.Zone).Format("15:04:05")
	}
	lines := []string{
		"",
		section("STATION"),
		reading("ocpp", draw.Dash(s.Ocpp)) + reading("link", draw.StatusStyle(s.Link).Render(draw.Dash(s.Link))),
		reading("boot", draw.StatusStyle(s.Boot).Render(draw.Dash(s.Boot))) + reading("since", since),
		reading("vendor", draw.Dash(d.Vendor)) + reading("model", draw.Dash(d.Model)),
		reading("firmware", draw.Dash(d.FirmwareVersion)),
		"  " + draw.Dim.Render(draw.Pad("csms", labelWidth)) + " " + draw.Dash(d.Csms),
		"",
		section("CONNECTORS"),
	}
	if len(s.Connectors) == 0 {
		lines = append(lines, "  "+draw.Dim.Render("none reported yet"))
	} else {
		lines = append(lines, "    "+draw.Dim.Render(draw.Pad("#", 4)+draw.Pad("status", 13)+draw.Pad("kW", 8)+draw.Pad("soc", 7)+draw.Pad("meter", 12)+"tx"))
	}
	for index, connector := range s.Connectors {
		lines = append(lines, connectorLine(connector, index == min(p.connector, len(s.Connectors)-1)))
	}
	lines = append(lines, "", section("RECENT"))
	for _, line := range c.Recent {
		lines = append(lines, "  "+draw.Dim.Render(line))
	}
	return append(lines, "")
}

func reading(label, value string) string {
	return "  " + draw.Dim.Render(draw.Pad(label, labelWidth)) + draw.Pad(" "+value, 24)
}

func connectorLine(c fleet.Connector, isSelected bool) string {
	marker, number := "  ", draw.Pad(fmt.Sprint(c.ID), 4)
	if isSelected {
		marker, number = draw.Accent.Render("▌ "), draw.Accent.Render(number)
	}
	kw, soc, meter := draw.Dim.Render(draw.Pad("—", 8)), draw.Dim.Render(draw.Pad("—", 7)), draw.Dim.Render(draw.Pad("—", 12))
	if c.Sampled {
		kw, soc = draw.Pad(fmt.Sprintf("%.1f", c.PowerW/1000), 8), draw.Pad(fmt.Sprintf("%.0f%%", c.Soc*100), 7)
	}
	if c.MeterWh > 0 {
		meter = draw.Pad(fmt.Sprintf("%.1f kWh", c.MeterWh/1000), 12)
	}
	return marker + "  " + number + draw.StatusStyle(c.Status).Render(draw.Pad(draw.Dash(c.Status), 13)) + kw + soc + meter + draw.Dash(c.Transaction)
}

func tabbed(station string, active int) string {
	names := []string{"details", "config"}
	for index, name := range names {
		if index == active {
			names[index] = draw.ActiveTab.Render(name)
		} else {
			names[index] = draw.Dim.Render(name)
		}
	}
	return draw.Bold.Render(station) + draw.Dim.Render(" ─ ") + strings.Join(names, draw.Dim.Render(" · "))
}

func (p Station) Bindings() []draw.Binding {
	if p.tab == configTab {
		return configBindings
	}
	return detailsBindings
}

func (p Station) Bar(width int) string {
	switch {
	case p.tab == configTab && p.config.draft != nil:
		return keyHints([][2]string{{"enter", "apply"}, {"esc", "cancel"}})
	case p.tab == configTab && p.config.filtering:
		return keyHints([][2]string{{"enter", "done"}, {"esc", "clear"}})
	}
	return draw.KeyBar(width, p.Bindings(), p.pending)
}
