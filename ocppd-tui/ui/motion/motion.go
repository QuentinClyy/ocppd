package motion

import (
	"strconv"
	"strings"
)

const countDigits = 4

func Feed(pending, key, prefixes string) (next string, count int, motion string) {
	digits := strings.TrimRight(pending, prefixes)
	prefix := pending[len(digits):]
	switch {
	case pending != "" && key == "esc":
		return "", 0, ""
	case prefix != "":
		return "", parse(digits), prefix + key
	case len(key) == 1 && strings.Contains(prefixes, key):
		return pending + key, 0, ""
	case isDigit(key) && (key != "0" || pending != ""):
		if len(pending) == countDigits {
			return pending, 0, ""
		}
		return pending + key, 0, ""
	}
	return "", parse(pending), key
}

func isDigit(key string) bool {
	return len(key) == 1 && key[0] >= '0' && key[0] <= '9'
}

func parse(digits string) int {
	count, err := strconv.Atoi(digits)
	if err != nil {
		return 0
	}
	return count
}

func Move(key string, count, index, last int) (int, bool) {
	steps := max(count, 1)
	switch key {
	case "j", "down":
		return min(index+steps, last), true
	case "k", "up":
		return max(index-steps, 0), true
	case "gg":
		return min(max(count-1, 0), last), true
	case "G":
		if count == 0 {
			return last, true
		}
		return min(count-1, last), true
	}
	return index, false
}
