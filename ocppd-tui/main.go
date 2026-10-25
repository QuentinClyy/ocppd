package main

import (
	"errors"
	"fmt"
	"os"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ocppd"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui"
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
	process, err := ocppd.Start(ocppd.Path(os.Getenv("OCPPD"), self), csms)
	if err != nil {
		fmt.Fprintf(os.Stderr, "ocppd-tui: cannot start ocppd: %v\n", err)
		os.Exit(1)
	}
	program := tea.NewProgram(ui.New(csms))
	process.Forward(program.Send)
	_, err = program.Run()
	process.Stop()
	if err != nil {
		fmt.Fprintf(os.Stderr, "ocppd-tui: %v\n", err)
		os.Exit(1)
	}
}
