#![feature(let_chains)]

use io::prelude::*;
use pi::PiCalc;
use std::{fs::File, io};

mod pi;

const PI_DIGITS: u32 = 1_000_000_000;

fn main() -> io::Result<()> {
    let mut pi = PiCalc::new();
    let mut f = File::create("./PI.TXT")?;
    write!(f, "{}", pi.get_pi(PI_DIGITS))?;

    Ok(())
}
