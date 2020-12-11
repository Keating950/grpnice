use std::{
    cmp::{max, min},
    collections::VecDeque,
    env,
    fs,
    io,
    num::ParseIntError,
    process::exit,
};

const HELP: &'static str = concat!(
    env!("CARGO_PKG_NAME"),
    ": Adjusts niceness for the given PID's process group.\n\n",
    "USAGE:\tgrpnice [-n (adj)] (PID)\n",
    "(adj):\tAdded to the process group's niceness. Must be an integer. Defaults to 10.\n",
    "(PID):\tPID to adjust.\n",
    "-h:\tPrint this help message.\n",
    "-v:\tPrint version info.",
);
const VERSION: &'static str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
    "\nCopyright (C) ",
    env!("CARGO_PKG_AUTHORS"),
    "\nReleased and distributed under the terms of the MIT licence.",
);

enum ArgError {
    PrintHelp,
    PrintVersion,
    InsufficientArgs,
    ParseError(ParseIntError),
    FileNotFound(io::Error),
}

impl From<ParseIntError> for ArgError {
    fn from(e: ParseIntError) -> Self {
        ArgError::ParseError(e)
    }
}
impl From<io::Error> for ArgError {
    fn from(e: io::Error) -> Self {
        ArgError::FileNotFound(e)
    }
}

macro_rules! die {
    ($s:expr) => { {eprintln!("{}", $s); exit(1);} };
    ($s:expr, $n:literal) => { {eprintln!("{}", $s); exit($n);} };
}

fn main() {
    let mut args: VecDeque<String> = env::args().skip(1).collect();
    let (pid, adjustment) = match parse_args(&mut args) {
        Err(ArgError::PrintHelp) => die!(HELP),
        Err(ArgError::PrintVersion) => die!(VERSION, 0),
        Err(ArgError::InsufficientArgs) => die!(HELP),
        Err(ArgError::ParseError(e)) => {
            eprintln!("Failed to parse argument {}", e);
            die!(HELP)
        }
        Ok((p, a)) => (p, a),
        _ => unreachable!(),
    };
    match renice(pid, adjustment) {
        Ok((grp, old, new)) =>
            println!("{} ({}): old priority {}, new priority {}", pid, grp, old, new),
        Err(ArgError::ParseError(e)) =>
            die!(format!("Invalid value in /proc/{}/autogroup: {}", pid, e)),
        Err(ArgError::FileNotFound(e)) => die!(e),
        _ => unreachable!(),
    };
}

fn parse_args(args: &mut VecDeque<String>) -> Result<(usize, i32), ArgError> {
    let mut adjustment: Option<i32> = None;
    let mut pid: Option<usize> = None;
    while pid.is_none() {
        match args.pop_front().as_deref() {
            None => return Err(ArgError::InsufficientArgs),
            Some("-h") => return Err(ArgError::PrintHelp),
            Some("-v") => return Err(ArgError::PrintVersion),
            Some("-n") => adjustment = Some(args.pop_front().expect(HELP).parse()?),
            Some(p) => pid = Some(p.parse::<usize>()?),
        }
    }
    Ok((pid.unwrap(), adjustment.unwrap_or(10)))
}

fn renice(pid: usize, adjustment: i32) -> Result<(String, i32, i32), ArgError> {
    let path = format!("/proc/{}/autogroup", pid);
    let contents = fs::read_to_string(&path)?;
    let mut fields: Vec<&str> = contents.split(" ").map(str::trim).collect();
    let niceness: i32 = fields.pop().unwrap().parse()?;
    let new_niceness = match niceness {
        n if adjustment >= 0 => min(n + adjustment, 19),
        n => max(n + adjustment, -20),
    };
    fs::write(path, new_niceness.to_string())?;
    Ok((fields[0].to_string(), niceness, new_niceness))
}
