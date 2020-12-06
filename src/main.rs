use std::{
    cmp::{max, min},
    collections::VecDeque,
    env, error, fs,
    process::exit,
};

type BoxResult<T> = Result<T, Box<dyn error::Error>>;
const HELP: &'static str = "
grpnice: Adjusts niceness for the given PID's process group.

USAGE:\tgrpnice [-n (adjustment)] (PID)
n:\tAdded to the process group's niceness. Must be an integer. Defaults to 10.
PID:\tPID to adjust.
-h:\tPrint this help message.
-v:\tPrint version info.
";

macro_rules! help_exit {
    ($n:literal) => { {eprintln!("{}", HELP); exit($n);} };
}

fn main() {
    let mut args: VecDeque<String> = env::args().skip(1).collect();
    let (pid, adjustment) = parse_args(&mut args).unwrap_or_else(|_| help_exit!(1));
    let (grp, old, new) = match renice(pid, adjustment) {
        Ok((grp, old, new)) => (grp, old, new),
        Err(msg) => {
            //  trait object erases error info, so string matching is necessary
            match msg.to_string() {
                s if s.starts_with("No such file") =>
                    eprintln!("No autogroup found for PID {}", pid),
                s => eprintln!("{}", s),
            }
            exit(1);
        }
    };
    println!("{} ({}): old priority {}, new priority {}", pid, grp, old, new);
}

fn parse_args(args: &mut VecDeque<String>) -> BoxResult<(usize, i32)> {
    let mut adjustment: Option<i32> = None;
    let mut pid: Option<usize> = None;
    while pid.is_none() {
        match args.pop_front().as_deref() {
            None => help_exit!(1),
            Some("-h") => help_exit!(0),
            Some("-v") => {
                print_version_info();
                exit(0)
            }
            Some("-n") =>
                adjustment = Some(args.pop_front().unwrap_or_else(|| help_exit!(1)).parse()?),
            Some(p) => {
                pid = Some(p.parse::<usize>()?);
            }
        }
    }
    Ok((pid.unwrap(), adjustment.unwrap_or(10)))
}

fn renice(pid: usize, adjustment: i32) -> BoxResult<(String, i32, i32)> {
    let path = format!("/proc/{}/autogroup", pid);
    let contents = fs::read_to_string(&path)?;
    let mut fields: Vec<&str> = contents.split(" ").map(str::trim).collect();
    let niceness: i32 = fields.pop().unwrap().parse()?;
    let new_niceness = match niceness {
        n if adjustment >= 0 => min(n + adjustment, 19),
        n => max(n - adjustment, -20),
    };
    let new_contents = format!("{} {}\n", fields.join(" "), &new_niceness.to_string());
    fs::write(&path, new_contents)?;
    Ok((fields[0].to_string(), niceness, new_niceness))
}

fn print_version_info() {
    println!(
        "{} {}\nCopyright (C) {}\nReleased and distributed under the terms of the MIT licence.",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    )
}
