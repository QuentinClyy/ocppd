package main

import (
	"errors"
	"testing"
)

func Test_given_a_csms_flag_when_parsed_then_its_url_without_a_trailing_slash(t *testing.T) {
	args := []string{"--csms", "ws://localhost:8081/"}

	csms, err := parseArgs(args, "")

	if err != nil || csms != "ws://localhost:8081" {
		t.Fatalf("got %q, %v", csms, err)
	}
}

func Test_given_an_equals_flag_and_an_env_url_when_parsed_then_the_flag_wins(t *testing.T) {
	args := []string{"--csms=ws://flag"}

	csms, err := parseArgs(args, "ws://env")

	if err != nil || csms != "ws://flag" {
		t.Fatalf("got %q, %v", csms, err)
	}
}

func Test_given_only_the_env_url_when_parsed_then_the_env_url(t *testing.T) {
	var args []string

	csms, err := parseArgs(args, "ws://env")

	if err != nil || csms != "ws://env" {
		t.Fatalf("got %q, %v", csms, err)
	}
}

func Test_given_no_url_when_parsed_then_a_missing_url_error(t *testing.T) {
	var args []string

	_, err := parseArgs(args, "")

	if err == nil || err.Error() != "missing --csms <url> (or OCPPD_CSMS)" {
		t.Fatalf("got %v", err)
	}
}

func Test_given_help_when_parsed_then_the_help_error(t *testing.T) {
	args := []string{"--csms", "ws://x", "-h"}

	_, err := parseArgs(args, "")

	if !errors.Is(err, errHelp) {
		t.Fatalf("got %v", err)
	}
}

func Test_given_an_unknown_argument_when_parsed_then_refused(t *testing.T) {
	args := []string{"--json"}

	_, err := parseArgs(args, "ws://env")

	if err == nil || err.Error() != "unknown argument '--json'" {
		t.Fatalf("got %v", err)
	}
}
