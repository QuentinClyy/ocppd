package ocppd

import (
	"bufio"
	"net"
	"path/filepath"
	"testing"
	"time"

	tea "charm.land/bubbletea/v2"
)

func server(t *testing.T, network string) (net.Listener, string) {
	address := "127.0.0.1:0"
	if network == "unix" {
		address = filepath.Join(t.TempDir(), "ocppd.sock")
	}
	listener, err := net.Listen(network, address)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { listener.Close() })
	return listener, network + ":" + listener.Addr().String()
}

func accept(t *testing.T, listener net.Listener) (net.Conn, *bufio.Scanner) {
	conn, err := listener.Accept()
	if err != nil {
		t.Fatal(err)
	}
	return conn, bufio.NewScanner(conn)
}

func next(t *testing.T, messages chan tea.Msg) tea.Msg {
	select {
	case msg := <-messages:
		return msg
	case <-time.After(5 * time.Second):
		t.Fatal("no message")
		return nil
	}
}

func Test_given_a_daemon_that_writes_then_closes_when_forwarded_then_its_lines_then_closed(t *testing.T) {
	listener, address := server(t, "unix")
	connection, err := Connect(address, "")
	if err != nil {
		t.Fatal(err)
	}
	conn, _ := accept(t, listener)
	messages := make(chan tea.Msg, 4)

	connection.Forward(func(msg tea.Msg) { messages <- msg })
	conn.Write([]byte(`{"kind":"daemon","csms":"ws://x","version":"0.2.0"}` + "\n"))
	conn.Close()

	if got := next(t, messages); got != EventMsg(`{"kind":"daemon","csms":"ws://x","version":"0.2.0"}`) {
		t.Fatalf("got %v", got)
	}
	if _, closed := next(t, messages).(ClosedMsg); !closed {
		t.Fatal("no closed message")
	}
}

func Test_given_a_connection_when_a_line_is_sent_then_the_daemon_reads_it(t *testing.T) {
	listener, address := server(t, "unix")
	connection, err := Connect(address, "")
	if err != nil {
		t.Fatal(err)
	}
	_, lines := accept(t, listener)

	err = connection.Send(`{"command":"shutdown"}`)

	if err != nil || !lines.Scan() || lines.Text() != `{"command":"shutdown"}` {
		t.Fatalf("got %q, %v", lines.Text(), err)
	}
}

func Test_given_tcp_with_a_token_when_connected_then_hello_is_the_first_line(t *testing.T) {
	listener, address := server(t, "tcp")

	_, err := Connect(address, "s3cret")
	_, lines := accept(t, listener)

	if err != nil || !lines.Scan() || lines.Text() != `{"command":"hello","token":"s3cret"}` {
		t.Fatalf("got %q, %v", lines.Text(), err)
	}
}

func Test_given_tcp_without_a_token_when_connecting_then_refused_before_dialing(t *testing.T) {
	address := "tcp:127.0.0.1:1"

	_, err := Connect(address, "")

	if err == nil || err.Error() != "connecting over tcp needs OCPPD_TOKEN" {
		t.Fatalf("got %v", err)
	}
}

func Test_given_an_address_without_a_scheme_when_connecting_then_refused(t *testing.T) {
	address := "/tmp/ocppd.sock"

	_, err := Connect(address, "")

	if err == nil {
		t.Fatal("connected")
	}
}

func Test_given_the_socket_variables_when_the_default_socket_is_chosen_then_in_their_order(t *testing.T) {
	cases := []struct {
		env  map[string]string
		want string
	}{
		{map[string]string{"OCPPD_SOCKET": "/s.sock", "XDG_RUNTIME_DIR": "/run/user/1", "HOME": "/home/q"}, "/s.sock"},
		{map[string]string{"XDG_RUNTIME_DIR": "/run/user/1", "HOME": "/home/q"}, "/run/user/1/ocppd.sock"},
		{map[string]string{"HOME": "/home/q"}, "/home/q/.ocppd.sock"},
	}

	for _, c := range cases {
		socket := DefaultSocket(func(key string) string { return c.env[key] })

		if socket != c.want {
			t.Fatalf("%v: got %q", c.env, socket)
		}
	}
}
