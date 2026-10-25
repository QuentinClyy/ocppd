package fleet

import (
	"strconv"
	"strings"
	"unicode"
)

func naturalCompare(a, b string) int {
	for a != "" && b != "" {
		headA, restA := run(a)
		headB, restB := run(b)
		numberA, errA := strconv.Atoi(headA)
		numberB, errB := strconv.Atoi(headB)
		switch {
		case errA == nil && errB == nil && numberA != numberB:
			return numberA - numberB
		case headA != headB:
			return strings.Compare(headA, headB)
		}
		a, b = restA, restB
	}
	return len(a) - len(b)
}

func run(text string) (string, string) {
	digit := unicode.IsDigit(rune(text[0]))
	end := strings.IndexFunc(text, func(r rune) bool { return unicode.IsDigit(r) != digit })
	if end < 0 {
		return text, ""
	}
	return text[:end], text[end:]
}
