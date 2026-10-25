pub const USAGE: &str = "usage: ocppd --csms <url> [--json]";

pub const HELP: &str = "\
ocppd - simulate OCPP 1.6 and 2.0.1 charging stations against a CSMS

usage: ocppd --csms <url> [--json]

In a terminal, opens the ocppd-tui interface when it is installed beside
ocppd or on the PATH. Otherwise, or with --json, reads commands from stdin,
one JSON object per line, and writes events to stdout, one JSON object per
line. Logs go to stderr.

options:
  --csms <url>     CSMS WebSocket url; a station connects to <url>/<id>
                   [env: OCPPD_CSMS]
  --json           read and write JSON lines even in a terminal
  -h, --help       print this help and exit
  -V, --version    print the version and exit

commands:
  {\"command\":\"spawn\",\"prefix\":\"CP\",\"count\":2}             start CP1 and CP2 (OCPP 1.6)
  {\"command\":\"spawn\",\"prefix\":\"CS\",\"count\":1,\"ocpp\":\"2.0.1\"}  start CS1 (OCPP 2.0.1)
  {\"command\":\"plug_in\",\"station\":\"CP1\",\"connector\":1}      plug an EV in
  {\"command\":\"authorize\",\"station\":\"CP1\",\"id_tag\":\"TAG1\"}   present a card
  {\"command\":\"stop\",\"station\":\"CP1\",\"connector\":1}         stop the transaction
  {\"command\":\"unplug\",\"station\":\"CP1\",\"connector\":1}       unplug the EV

example:
  ocppd --csms ws://localhost:8180/steve/websocket/CentralSystemService

exit status:
  0  stdin closed and every station stopped
  1  runtime error
  2  usage error

docs: https://github.com/QuentinClyy/ocppd/tree/main/docs
";

#[derive(Debug, PartialEq)]
pub enum Cli {
    Run { csms: String, json: bool },
    Help,
    Version,
}

pub fn parse(args: &[String], env_csms: Option<String>) -> Result<Cli, String> {
    let mut csms = None;
    let mut json = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Cli::Help),
            "-V" | "--version" => return Ok(Cli::Version),
            "--json" => json = true,
            "--csms" => match args.next() {
                Some(url) => csms = Some(url.clone()),
                None => return Err("--csms needs a url".into()),
            },
            other => match other.strip_prefix("--csms=") {
                Some(url) => csms = Some(url.to_owned()),
                None => return Err(format!("unknown argument '{other}'")),
            },
        }
    }
    let csms = csms
        .or(env_csms)
        .filter(|url| !url.is_empty())
        .ok_or("missing --csms <url> (or OCPPD_CSMS)")?;
    Ok(Cli::Run {
        csms: csms.trim_end_matches('/').to_owned(),
        json,
    })
}

#[cfg(test)]
#[path = "cli.test.rs"]
mod tests;
