package ui

import "strings"

func textField(text, hint string, width int) string {
	if text == "" {
		return fieldHint.Render(pad(" "+hint, width))
	}
	return field.Render(pad(" "+tail(text, width-2), width))
}

func editingField(text string, width int) string {
	shown := tail(text, width-2)
	return field.Render(" "+shown) + cursorBlock.Render(" ") + field.Render(pad("", width-2-len([]rune(shown))))
}

func statusLine(badge, err string) string {
	if err == "" {
		return badge
	}
	return badge + " " + warn.Render("! "+err)
}

func checkbox(on bool, label string) string {
	if on {
		return dim.Render(" [") + accent.Render("x") + dim.Render("] ") + label
	}
	return dim.Render(" [ ] " + label)
}

func segmented(c choice, focused bool) string {
	parts := make([]string, len(c.options))
	for index, option := range c.options {
		style := dim
		switch {
		case index == c.index && focused:
			style = chosen
		case index == c.index:
			style = selected
		}
		parts[index] = style.Render(" " + option + " ")
	}
	return strings.Join(parts, dim.Render("│"))
}

func section(title string) string {
	return "  " + heading.Render(title)
}

func keyHints(keys [][2]string) string {
	var items []string
	for _, key := range keys {
		items = append(items, accent.Render(key[0])+" "+key[1])
	}
	return " " + strings.Join(items, "  ")
}
