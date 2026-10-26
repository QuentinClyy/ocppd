package main

import (
	"errors"
	"fmt"
	"os"

	tea "charm.land/bubbletea/v2"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ocppd"
	"github.com/QuentinClyy/ocppd/ocppd-tui/ui"
)

const usage = "usage: ocppd-tui [--connect unix:<path>|tcp:<host>:<port>]"

func main() {
	address, err := parseArgs(os.Args[1:], ocppd.DefaultSocket(os.Getenv))
	if errors.Is(err, errHelp) {
		fmt.Println(usage)
		return
	}
	if err != nil {
		fmt.Fprintf(os.Stderr, "ocppd-tui: %v\n%s\n", err, usage)
		os.Exit(2)
	}
	connection, err := ocppd.Connect(address, os.Getenv("OCPPD_TOKEN"))
	if err != nil {
		fmt.Fprintf(os.Stderr, "ocppd-tui: cannot connect to %s: %v\n", address, err)
		os.Exit(1)
	}
	program := tea.NewProgram(ui.New(connection.Send))
	connection.Forward(program.Send)
	final, err := program.Run()
	connection.Close()
	if err != nil {
		fmt.Fprintf(os.Stderr, "ocppd-tui: %v\n", err)
		os.Exit(1)
	}
	if final.(ui.Model).Lost() {
		fmt.Fprintln(os.Stderr, "ocppd-tui: the daemon closed the connection")
		os.Exit(1)
	}
}
