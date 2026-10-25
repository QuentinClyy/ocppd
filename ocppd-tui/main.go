package main

import (
	"errors"
	"fmt"
	"os"

	tea "charm.land/bubbletea/v2"
)

const usage = "usage: ocppd-tui --csms <url>"

func main() {
	csms, err := parseArgs(os.Args[1:], os.Getenv("OCPPD_CSMS"))
	if errors.Is(err, errHelp) {
		fmt.Println(usage)
		return
	}
	if err != nil {
		fmt.Fprintf(os.Stderr, "ocppd-tui: %v\n%s\n", err, usage)
		os.Exit(2)
	}
	self, err := os.Executable()
	if err != nil {
		self = ""
	}
	child, err := start(ocppdPath(os.Getenv("OCPPD"), self), csms)
	if err != nil {
		fmt.Fprintf(os.Stderr, "ocppd-tui: cannot start ocppd: %v\n", err)
		os.Exit(1)
	}
	program := tea.NewProgram(newModel(csms))
	child.forward(program.Send)
	_, err = program.Run()
	child.stop()
	if err != nil {
		fmt.Fprintf(os.Stderr, "ocppd-tui: %v\n", err)
		os.Exit(1)
	}
}
