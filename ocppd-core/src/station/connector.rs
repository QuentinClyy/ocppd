use super::charge::{self, Charging, Current, Ev, Sample};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Connector {
    pub current: Current,
    pub register_wh: f64,
    pub plugged: Option<Plugged>,
    pub fault: Option<Fault>,
    pub lock_broken: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fault {
    Internal,
    ConnectorLock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plugged {
    pub ev: Ev,
    pub soc: f64,
    pub charging: Option<Charging>,
}

impl Connector {
    pub fn plug_in(&mut self, ev: Ev, soc: f64) {
        self.plugged = Some(Plugged {
            ev,
            soc,
            charging: None,
        });
        if self.lock_broken {
            self.fault = Some(Fault::ConnectorLock);
        }
    }

    pub fn repair(&mut self) {
        self.fault = None;
        self.lock_broken = false;
    }

    pub fn unplug(&mut self, now: u64) {
        self.stop_charging(now);
        self.plugged = None;
    }

    pub fn start_charging(&mut self, now: u64, limit_w: f64) {
        let (current, register_wh) = (self.current, self.register_wh);
        if let Some(plugged) = self.plugged.as_mut() {
            plugged.charging = Some(Charging {
                current,
                start: now,
                start_soc: plugged.soc,
                start_register_wh: register_wh,
                limit_w,
            });
        }
    }

    pub fn limit_to(&mut self, now: u64, limit_w: f64) {
        if let Some(plugged) = self.plugged.as_mut()
            && let Some(charging) = plugged.charging.as_mut()
        {
            *charging = charge::reanchor(charging, &plugged.ev, now, limit_w);
        }
    }

    pub fn stop_charging(&mut self, now: u64) {
        let at_now = self.sample(now);
        if let Some(plugged) = self.plugged.as_mut() {
            plugged.soc = at_now.soc;
            plugged.charging = None;
        }
        self.register_wh = at_now.register_wh;
    }

    pub fn sample(&self, now: u64) -> Sample {
        let idle = |soc| Sample {
            register_wh: self.register_wh,
            power_w: 0.0,
            soc,
        };
        match &self.plugged {
            Some(Plugged {
                ev,
                charging: Some(charging),
                ..
            }) => charge::sample(charging, ev, now),
            Some(plugged) => idle(plugged.soc),
            None => idle(0.0),
        }
    }

    pub fn offered_w(&self) -> Option<f64> {
        let charging = self.plugged.as_ref()?.charging.as_ref()?;
        Some(charging.limit_w)
    }

    pub fn finish_time(&self) -> Option<u64> {
        let plugged = self.plugged.as_ref()?;
        charge::finish_time(plugged.charging.as_ref()?, &plugged.ev)
    }
}
