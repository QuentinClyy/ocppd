package main

import (
	"errors"
	"fmt"
	"strings"
)

var errHelp = errors.New("help")

func parseArgs(args []string, envCsms string) (string, error) {
	csms := envCsms
	for i := 0; i < len(args); i++ {
		switch arg := args[i]; {
		case arg == "-h" || arg == "--help":
			return "", errHelp
		case arg == "--csms":
			if i+1 == len(args) {
				return "", errors.New("--csms needs a url")
			}
			i++
			csms = args[i]
		case strings.HasPrefix(arg, "--csms="):
			csms = strings.TrimPrefix(arg, "--csms=")
		default:
			return "", fmt.Errorf("unknown argument '%s'", arg)
		}
	}
	if csms == "" {
		return "", errors.New("missing --csms <url> (or OCPPD_CSMS)")
	}
	return strings.TrimRight(csms, "/"), nil
}
