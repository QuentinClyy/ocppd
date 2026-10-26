package ocppd

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"path/filepath"
	"strings"

	tea "charm.land/bubbletea/v2"
)

type EventMsg string

type ClosedMsg struct{ Err error }

type Connection struct {
	conn net.Conn
}

func DefaultSocket(env func(string) string) string {
	if socket := env("OCPPD_SOCKET"); socket != "" {
		return socket
	}
	if runtime := env("XDG_RUNTIME_DIR"); runtime != "" {
		return filepath.Join(runtime, "ocppd.sock")
	}
	return filepath.Join(env("HOME"), ".ocppd.sock")
}

func Connect(address, token string) (*Connection, error) {
	network, target, ok := strings.Cut(address, ":")
	if !ok || target == "" || network != "unix" && network != "tcp" {
		return nil, fmt.Errorf("address '%s' must be unix:<path> or tcp:<host>:<port>", address)
	}
	if network == "tcp" && token == "" {
		return nil, fmt.Errorf("connecting over tcp needs OCPPD_TOKEN")
	}
	conn, err := net.Dial(network, target)
	if err != nil {
		return nil, err
	}
	c := &Connection{conn}
	if network == "tcp" {
		hello, _ := json.Marshal(map[string]string{"command": "hello", "token": token})
		if err := c.Send(string(hello)); err != nil {
			conn.Close()
			return nil, err
		}
	}
	return c, nil
}

func (c *Connection) Forward(send func(tea.Msg)) {
	go func() {
		send(ClosedMsg{readLines(c.conn, send)})
	}()
}

func (c *Connection) Send(line string) error {
	_, err := io.WriteString(c.conn, line+"\n")
	return err
}

func (c *Connection) Close() {
	c.conn.Close()
}

func readLines(reader io.Reader, send func(tea.Msg)) error {
	lines := bufio.NewScanner(reader)
	lines.Buffer(make([]byte, 64*1024), 1024*1024)
	for lines.Scan() {
		send(EventMsg(lines.Text()))
	}
	return lines.Err()
}
