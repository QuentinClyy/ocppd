package main

import (
	"errors"
	"fmt"
	"strings"
)

var errHelp = errors.New("help")

func parseArgs(args []string, defaultSocket string) (string, error) {
	address := "unix:" + defaultSocket
	for i := 0; i < len(args); i++ {
		switch arg := args[i]; {
		case arg == "-h" || arg == "--help":
			return "", errHelp
		case arg == "--connect":
			if i+1 == len(args) {
				return "", errors.New("--connect needs an address")
			}
			i++
			address = args[i]
		case strings.HasPrefix(arg, "--connect="):
			address = strings.TrimPrefix(arg, "--connect=")
		default:
			return "", fmt.Errorf("unknown argument '%s'", arg)
		}
	}
	return address, nil
}
