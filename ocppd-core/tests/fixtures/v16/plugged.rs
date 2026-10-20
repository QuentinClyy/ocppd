use crate::fixtures::v16::fake_csms::Csms;
use crate::intent::{EvSpec, Intent};

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
