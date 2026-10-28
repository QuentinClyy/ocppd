package ui

import (
	"strings"
	"testing"

	tea "charm.land/bubbletea/v2"
)

func Test_given_a_selected_connector_when_an_id_tag_is_typed_and_entered_then_authorize_is_sent(t *testing.T) {
	m, sent := recording()

	press(m, "j", "a", "T", "A", "G", "9", "backspace", "1", "enter")

	if len(*sent) != 1 || (*sent)[0] != `{"command":"authorize","connector":1,"id_tag":"TAG1","station":"CP1"}` {
		t.Fatalf("got %v", *sent)
	}
}

func Test_given_the_id_tag_prompt_when_esc_then_nothing_is_sent_and_keys_drive_again(t *testing.T) {
	m, sent := recording()

	m = press(m, "j", "a", "T", "esc", "p")

	if m.prompt != nil || len(*sent) != 1 || !strings.Contains((*sent)[0], "plug_in") {
		t.Fatalf("prompt %v, sent %v", m.prompt, *sent)
	}
}

func Test_given_the_id_tag_prompt_when_q_is_typed_then_it_is_part_of_the_tag(t *testing.T) {
	m, _ := recording()

	m = press(m, "j", "a", "q")

	if m.prompt == nil || m.prompt.value.String() != "q" {
		t.Fatalf("got %+v", m.prompt)
	}
}

func Test_given_the_id_tag_prompt_when_text_is_pasted_then_it_is_part_of_the_tag(t *testing.T) {
	m, sent := recording()
	m = press(m, "j", "a")

	next, _ := m.Update(tea.PasteMsg{Content: "TAG\nq"})
	press(next.(Model), "enter")

	if len(*sent) != 1 || !strings.Contains((*sent)[0], `"id_tag":"TAGq"`) {
		t.Fatalf("got %v", *sent)
	}
}
