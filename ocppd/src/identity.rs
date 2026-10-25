use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Identity {
    pub id: String,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct Identities {
    stations: Option<Vec<Identity>>,
    prefix: Option<String>,
    count: Option<u32>,
    password: Option<String>,
}

impl Identities {
    pub fn list(&self) -> Result<Vec<Identity>, &'static str> {
        if let Some(stations) = &self.stations {
            if self.prefix.is_some() || self.count.is_some() || self.password.is_some() {
                return Err("stations takes no prefix, count or password");
            }
            return Ok(stations.clone());
        }
        let (Some(prefix), Some(count)) = (&self.prefix, self.count) else {
            return Err("needs stations, or prefix with count");
        };
        let numbered = (1..=count).map(|n| Identity {
            id: format!("{prefix}{n}"),
            password: self.password.clone(),
        });
        Ok(numbered.collect())
    }
}

#[cfg(test)]
#[path = "identity.test.rs"]
mod tests;
