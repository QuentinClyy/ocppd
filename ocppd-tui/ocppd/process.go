package ocppd

import (
	"bufio"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sync"

	tea "charm.land/bubbletea/v2"
)

type EventMsg string

type ErrorMsg string

type ExitMsg struct{ Err error }

type Process struct {
	command *exec.Cmd
	stdin   io.WriteCloser
	stdout  io.Reader
	stderr  io.Reader
}

func Path(env, self string) string {
	if env != "" {
		return env
	}
	beside := filepath.Join(filepath.Dir(self), "ocppd")
	if info, err := os.Stat(beside); self != "" && err == nil && !info.IsDir() {
		return beside
	}
	return "ocppd"
}

func Start(path, csms string) (*Process, error) {
	command := exec.Command(path, "--csms", csms)
	stdin, err := command.StdinPipe()
	if err != nil {
		return nil, err
	}
	stdout, err := command.StdoutPipe()
	if err != nil {
		return nil, err
	}
	stderr, err := command.StderrPipe()
	if err != nil {
		return nil, err
	}
	if err := command.Start(); err != nil {
		return nil, err
	}
	return &Process{command, stdin, stdout, stderr}, nil
}

func (p *Process) Forward(send func(tea.Msg)) {
	var readers sync.WaitGroup
	readers.Add(2)
	go func() {
		readLines(p.stdout, func(line string) tea.Msg { return EventMsg(line) }, send)
		readers.Done()
	}()
	go func() {
		readLines(p.stderr, func(line string) tea.Msg { return ErrorMsg(line) }, send)
		readers.Done()
	}()
	go func() {
		readers.Wait()
		send(ExitMsg{p.command.Wait()})
	}()
}

func (p *Process) Stop() {
	p.stdin.Close()
	p.command.Process.Kill()
}

func readLines(reader io.Reader, wrap func(string) tea.Msg, send func(tea.Msg)) {
	lines := bufio.NewScanner(reader)
	lines.Buffer(make([]byte, 64*1024), 1024*1024)
	for lines.Scan() {
		send(wrap(lines.Text()))
	}
	if err := lines.Err(); err != nil {
		send(ErrorMsg("read: " + err.Error()))
	}
}
