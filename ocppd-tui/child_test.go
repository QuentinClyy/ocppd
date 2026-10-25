package main

import (
	"os"
	"path/filepath"
	"testing"
	"time"

	tea "charm.land/bubbletea/v2"
)

const fakeOcppd = `#!/bin/sh
echo "$@" >&2
echo '{"station":"CP1","at":0,"kind":"connected"}'
`

func Test_given_a_child_that_prints_when_forwarded_then_its_lines_then_its_exit_arrive(t *testing.T) {
	path := filepath.Join(t.TempDir(), "ocppd")
	if err := os.WriteFile(path, []byte(fakeOcppd), 0o755); err != nil {
		t.Fatal(err)
	}
	child, err := start(path, "ws://x")
	if err != nil {
		t.Fatal(err)
	}
	messages := make(chan tea.Msg, 8)

	child.forward(func(msg tea.Msg) { messages <- msg })

	got := map[tea.Msg]bool{}
	for {
		select {
		case msg := <-messages:
			if exit, ok := msg.(exitMsg); ok {
				if exit.err != nil || !got[eventMsg(`{"station":"CP1","at":0,"kind":"connected"}`)] || !got[errorMsg("--csms ws://x")] {
					t.Fatalf("exit %v after %v", exit.err, got)
				}
				return
			}
			got[msg] = true
		case <-time.After(5 * time.Second):
			t.Fatalf("no exit after %v", got)
		}
	}
}

func Test_given_ocppd_beside_the_tui_when_located_then_that_one(t *testing.T) {
	dir := t.TempDir()
	beside := filepath.Join(dir, "ocppd")
	if err := os.WriteFile(beside, nil, 0o755); err != nil {
		t.Fatal(err)
	}

	path := ocppdPath("", filepath.Join(dir, "ocppd-tui"))

	if path != beside {
		t.Fatalf("got %q", path)
	}
}

func Test_given_the_ocppd_env_when_located_then_the_env_wins(t *testing.T) {
	self := filepath.Join(t.TempDir(), "ocppd-tui")

	path := ocppdPath("/opt/ocppd", self)

	if path != "/opt/ocppd" {
		t.Fatalf("got %q", path)
	}
}

func Test_given_nothing_beside_the_tui_when_located_then_ocppd_from_the_path(t *testing.T) {
	self := filepath.Join(t.TempDir(), "ocppd-tui")

	path := ocppdPath("", self)

	if path != "ocppd" {
		t.Fatalf("got %q", path)
	}
}
