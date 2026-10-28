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
	case prefix != "" && key == prefix:
		return "", parse(digits), prefix + key
	case prefix != "":
		return "", 0, ""
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
