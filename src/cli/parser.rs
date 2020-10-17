use anyhow::Result;
use clap::{ArgMatches, Values};
use icore::arg::{Arg, SendArg, RecvArg};

pub fn parse_input(m: &ArgMatches) -> Result<Arg> {
    let arg = match (m.occurrences_of("send"), m.occurrences_of("receive")) {
        (1, 0) => parse_send_arg(m)?,
        (0, 1) => parse_recv_arg(m)?,
        _ => return Err(anyhow::Error::new(std::io::Error::new(std::io::ErrorKind::Other, ""))),
    };

    Ok(arg)
}
