use crate::api::event::LinkEvent;
use crate::v201::messages::availability::ConnectorStatus;
use crate::v201::messages::provisioning::boot::RegistrationStatus;
use crate::v201::messages::provisioning::reset::ResetType;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    BootStatus {
        status: RegistrationStatus,
    },
    StatusChanged {
        evse: u32,
        connector: u32,
        status: ConnectorStatus,
    },
    VariableChanged {
        component: String,
        variable: String,
        value: String,
    },
    Reset {
        reset_type: ResetType,
    },
    #[serde(untagged)]
    Link(LinkEvent),
}
