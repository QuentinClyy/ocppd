package main

import (
	"bufio"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sync"

	tea "charm.land/bubbletea/v2"
)

type eventMsg string

type errorMsg string

type exitMsg struct{ err error }

type child struct {
	process *exec.Cmd
	stdin   io.WriteCloser
	stdout  io.Reader
	stderr  io.Reader
}

func ocppdPath(env, self string) string {
	if env != "" {
		return env
	}
	beside := filepath.Join(filepath.Dir(self), "ocppd")
	if info, err := os.Stat(beside); self != "" && err == nil && !info.IsDir() {
		return beside
	}
	return "ocppd"
}

func start(path, csms string) (*child, error) {
	process := exec.Command(path, "--csms", csms)
	stdin, err := process.StdinPipe()
	if err != nil {
		return nil, err
	}
	stdout, err := process.StdoutPipe()
	if err != nil {
		return nil, err
	}
	stderr, err := process.StderrPipe()
	if err != nil {
		return nil, err
	}
	if err := process.Start(); err != nil {
		return nil, err
	}
	return &child{process, stdin, stdout, stderr}, nil
}

func (c *child) forward(send func(tea.Msg)) {
	var readers sync.WaitGroup
	readers.Add(2)
	go func() {
		readLines(c.stdout, func(line string) tea.Msg { return eventMsg(line) }, send)
		readers.Done()
	}()
	go func() {
		readLines(c.stderr, func(line string) tea.Msg { return errorMsg(line) }, send)
		readers.Done()
	}()
	go func() {
		readers.Wait()
		send(exitMsg{c.process.Wait()})
	}()
}

func (c *child) stop() {
	c.stdin.Close()
	c.process.Process.Kill()
}

func readLines(reader io.Reader, wrap func(string) tea.Msg, send func(tea.Msg)) {
	lines := bufio.NewScanner(reader)
	lines.Buffer(make([]byte, 64*1024), 1024*1024)
	for lines.Scan() {
		send(wrap(lines.Text()))
	}
	if err := lines.Err(); err != nil {
		send(errorMsg("read: " + err.Error()))
	}
}
