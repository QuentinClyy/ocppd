package popup

import "github.com/QuentinClyy/ocppd/ocppd-tui/ui/draw"

var detailsBindings = []draw.Binding{
	{Group: "connectors", Keys: "j / k", Help: "next / previous connector", Bar: "connector"},
	{Group: "connectors", Keys: "gg / G", Help: "first / last connector"},
	{Group: "driver", Keys: "p", Help: "plug in an EV", Bar: "plug"},
	{Group: "driver", Keys: "a", Help: "present an id tag", Bar: "auth"},
	{Group: "driver", Keys: "s", Help: "stop the transaction", Bar: "stop tx"},
	{Group: "driver", Keys: "u", Help: "unplug", Bar: "unplug"},
	{Group: "driver", Keys: "f", Help: "fault the connector"},
	{Group: "driver", Keys: "c", Help: "clear the fault"},
	{Group: "driver", Keys: "x", Help: "power cut, then reboot"},
	{Group: "driver", Keys: "o", Help: "take offline / bring online", Bar: "link"},
	{Group: "popup", Keys: "tab / gt", Help: "config tab", Bar: "config"},
	{Group: "popup", Keys: "esc / q", Help: "close", Bar: "close"},
}

var configBindings = []draw.Binding{
	{Group: "keys", Keys: "j / k", Help: "next / previous key", Bar: "key"},
	{Group: "keys", Keys: "gg / G", Help: "first / last key"},
	{Group: "keys", Keys: "/", Help: "filter the keys", Bar: "filter"},
	{Group: "keys", Keys: "space", Help: "flip a true / false value", Bar: "toggle"},
	{Group: "keys", Keys: "i / enter", Help: "edit the value", Bar: "edit"},
	{Group: "popup", Keys: "tab / gt", Help: "details tab", Bar: "details"},
	{Group: "popup", Keys: "esc / q", Help: "close", Bar: "close"},
}

var spawnBindings = []draw.Binding{
	{Group: "fields", Keys: "j / k", Help: "next / previous field", Bar: "field"},
	{Group: "fields", Keys: "gg / G", Help: "first / last field"},
	{Group: "fields", Keys: "h / l", Help: "switch a choice", Bar: "choice"},
	{Group: "fields", Keys: "i / a", Help: "type in the field", Bar: "edit"},
	{Group: "fields", Keys: "ctrl+n", Help: "add a station"},
	{Group: "form", Keys: "enter", Help: "spawn", Bar: "spawn"},
	{Group: "form", Keys: "esc / q", Help: "cancel", Bar: "cancel"},
}
