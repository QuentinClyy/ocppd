use crate::api::intent::{EvSpec, Intent};
use crate::fixtures::v16::fake_csms::Csms;

pub fn plugged() -> Csms {
    let mut csms = Csms::booted();
    csms.intent(
        0,
        Intent::PlugIn {
            connector: 1,
            ev: EvSpec::default(),
        },
    );
    csms
}
