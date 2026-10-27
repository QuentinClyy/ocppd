package ui

import "github.com/charmbracelet/x/ansi"

func follow(top, cursor, rows, total int) int {
	top = min(max(top, 0), max(total-rows, 0))
	if cursor < top {
		top = cursor
	}
	if cursor >= top+rows {
		top = cursor - rows + 1
	}
	return max(top, 0)
}

func window(lines []string, top, height int) []string {
	top = min(top, len(lines))
	return lines[top:min(top+height, len(lines))]
}

func tailLines(lines []string, height, offset int) []string {
	end := max(len(lines)-offset, 0)
	shown := lines[max(end-max(height, 0), 0):end]
	out := make([]string, len(shown))
	for index, line := range shown {
		out[index] = " " + line
	}
	return out
}

func fill(lines []string, height int) []string {
	for len(lines) < height {
		lines = append(lines, "")
	}
	return lines
}

func thumb(total, rows, top int) (int, int) {
	if total <= rows || rows == 0 {
		return 0, 0
	}
	size := max(rows*rows/total, 1)
	start := top * (rows - size) / (total - rows)
	return start, start + size
}

func scrollbar(lines []string, width, total, top int, track string) []string {
	start, end := thumb(total, len(lines), top)
	for index, line := range lines {
		mark := track
		if index >= start && index < end {
			mark = accent.Render("▐")
		}
		lines[index] = pad(ansi.Truncate(line, width-1, ""), width-1) + mark
	}
	return lines
}
