package main

import (
	"errors"
	"testing"
)

func Test_given_no_arguments_when_parsed_then_the_default_socket(t *testing.T) {
	var args []string

	address, err := parseArgs(args, "/run/user/1000/ocppd.sock")

	if err != nil || address != "unix:/run/user/1000/ocppd.sock" {
		t.Fatalf("got %q, %v", address, err)
	}
}

func Test_given_a_connect_flag_when_parsed_then_its_address(t *testing.T) {
	cases := [][]string{{"--connect", "tcp:127.0.0.1:7600"}, {"--connect=tcp:127.0.0.1:7600"}}

	for _, args := range cases {
		address, err := parseArgs(args, "/run/user/1000/ocppd.sock")

		if err != nil || address != "tcp:127.0.0.1:7600" {
			t.Fatalf("%v: got %q, %v", args, address, err)
		}
	}
}

func Test_given_connect_without_an_address_when_parsed_then_refused(t *testing.T) {
	args := []string{"--connect"}

	_, err := parseArgs(args, "/tmp/ocppd.sock")

	if err == nil || err.Error() != "--connect needs an address" {
		t.Fatalf("got %v", err)
	}
}

func Test_given_help_when_parsed_then_the_help_error(t *testing.T) {
	args := []string{"-h"}

	_, err := parseArgs(args, "/tmp/ocppd.sock")

	if !errors.Is(err, errHelp) {
		t.Fatalf("got %v", err)
	}
}

func Test_given_an_unknown_argument_when_parsed_then_refused(t *testing.T) {
	args := []string{"--csms", "ws://x"}

	_, err := parseArgs(args, "/tmp/ocppd.sock")

	if err == nil || err.Error() != "unknown argument '--csms'" {
		t.Fatalf("got %v", err)
	}
}
