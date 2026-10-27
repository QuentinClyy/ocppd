package draw

func TextField(text, hint string, width int) string {
	if text == "" {
		return FieldHint.Render(Pad(" "+hint, width))
	}
	return Field.Render(Pad(" "+Tail(text, width-2), width))
}

func EditingField(e Editor, width int) string {
	before, under, after := e.Window(width - 2)
	return Field.Render(" "+before) + CursorBlock.Render(under) + Field.Render(Pad(after, width-2-len([]rune(before))))
}

func Checkbox(on bool, label string) string {
	if on {
		return Dim.Render(" [") + Accent.Render("x") + Dim.Render("] ") + label
	}
	return Dim.Render(" [ ] " + label)
}
