package ui

import (
	"fmt"
	"strings"
	"time"

	"github.com/QuentinClyy/ocppd/ocppd-tui/fleet"
)

const (
	detailsWidth = 78
	recentLines  = 3
)

func (m Model) detailsLines() []string {
	s, _ := m.detailed()
	d := m.details.described
	since := dim.Render("—")
	if s.Since != 0 {
		since = time.UnixMilli(s.Since).In(m.zone).Format("15:04:05")
	}
	lines := []string{
		"",
		section("STATION"),
		reading("ocpp", dash(s.Ocpp)) + reading("link", statusStyle(s.Link).Render(dash(s.Link))),
		reading("boot", statusStyle(s.Boot).Render(dash(s.Boot))) + reading("since", since),
		reading("vendor", dash(d.Vendor)) + reading("model", dash(d.Model)),
		reading("firmware", dash(d.FirmwareVersion)),
		"  " + dim.Render(pad("csms", labelWidth)) + " " + dash(d.Csms),
		"",
		section("CONNECTORS"),
	}
	if len(s.Connectors) == 0 {
		lines = append(lines, "  "+dim.Render("none reported yet"))
	} else {
		lines = append(lines, "    "+dim.Render(pad("#", 4)+pad("status", 13)+pad("kW", 8)+pad("soc", 7)+pad("meter", 12)+"tx"))
	}
	for index, c := range s.Connectors {
		lines = append(lines, connectorLine(c, index == min(m.details.connector, len(s.Connectors)-1)))
	}
	lines = append(lines, "", section("RECENT"))
	var recent []string
	for index := len(m.log) - 1; index >= 0 && len(recent) < recentLines; index-- {
		if m.log[index].station == s.ID {
			recent = append([]string{"  " + dim.Render(m.log[index].withoutStation())}, recent...)
		}
	}
	lines = append(lines, recent...)
	return append(lines, "")
}

func reading(label, value string) string {
	return "  " + dim.Render(pad(label, labelWidth)) + pad(" "+value, 24)
}

func connectorLine(c fleet.Connector, isSelected bool) string {
	marker, number := "  ", pad(fmt.Sprint(c.ID), 4)
	if isSelected {
		marker, number = accent.Render("▌ "), accent.Render(number)
	}
	kw, soc, meter := dim.Render(pad("—", 8)), dim.Render(pad("—", 7)), dim.Render(pad("—", 12))
	if c.Sampled {
		kw, soc = pad(fmt.Sprintf("%.1f", c.PowerW/1000), 8), pad(fmt.Sprintf("%.0f%%", c.Soc*100), 7)
	}
	if c.MeterWh > 0 {
		meter = pad(fmt.Sprintf("%.1f kWh", c.MeterWh/1000), 12)
	}
	return marker + "  " + number + statusStyle(c.Status).Render(pad(dash(c.Status), 13)) + kw + soc + meter + dash(c.Transaction)
}

func (m Model) detailsPopup() popup {
	d := m.details
	p := popup{title: tabbed(d.station, d.tab), width: detailsWidth}
	if d.tab == detailsTab {
		p.lines = m.detailsLines()
		return p
	}
	lines, focused := d.configLines()
	p.lines, p.top, p.footer = lines, focused-m.modalRows()+1, d.config.footer()
	return p
}

func tabbed(station string, active int) string {
	names := []string{"details", "config"}
	for index, name := range names {
		if index == active {
			names[index] = activeTab.Render(name)
		} else {
			names[index] = dim.Render(name)
		}
	}
	return bold.Render(station) + dim.Render(" ─ ") + strings.Join(names, dim.Render(" · "))
}

func (d details) bar() string {
	switch {
	case d.tab == detailsTab:
		return keyHints([][2]string{{"j/k", "connector"}, {"p", "plug"}, {"a", "auth"}, {"s", "stop tx"}, {"u", "unplug"}, {"tab", "config"}, {"esc", "close"}})
	case d.config.draft != nil:
		return keyHints([][2]string{{"enter", "apply"}, {"esc", "cancel"}})
	case d.config.filtering:
		return keyHints([][2]string{{"enter", "done"}, {"esc", "clear"}})
	}
	return keyHints([][2]string{{"j/k", "key"}, {"/", "filter"}, {"space", "toggle"}, {"i", "edit"}, {"tab", "details"}, {"esc", "close"}})
}
