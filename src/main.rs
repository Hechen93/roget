//3:12:05

use clap::{clap_derive::ArgEnum, Parser};
use roget::Guesser;

const GAMES: &str = include_str!("../answers.txt");
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long, arg_enum)]
    implementation: Implementation,

    #[clap(short, long)]
    max: Option<usize>,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Implementation {
    Naive,
    Allocs,
}

impl std::str::FromStr for Implementation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "naive" => Ok(Self::Naive),
            _ => Err(format!("unknown implementation '{}'", s)),
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.implementation {
        Implementation::Naive => play(roget::algorithms::Naive::new, args.max),
        Implementation::Allocs => play(roget::algorithms::Allocs::new, args.max),
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = roget::Wordle::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
        if let Some(score) = w.play(answer, guesser) {
            println!("guessed '{}' in {}", answer, score);
        } else {
            eprintln!("failed to guess");
        }
    }
}
