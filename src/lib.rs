//Video at 3:20:00 - Stopped coding

use std::borrow::Cow;
use std::collections::HashSet;

pub mod algorithms;

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("every line is word + space + frequency")
                    .0
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        //play six rounds where it invokes the guesser each round
        let mut history = Vec::new();
        //Wordle only allows six guesses, we allow more to avoid chopping off score distribtion
        //For stats purposes
        for i in 1..=32 {
            let guess = guesser.guess(&history);
            if guess == answer {
                return Some(i);
            }
            assert!(
                self.dictionary.contains(&*guess),
                "guess '{}' is not in the dictionary",
                guess
            );
            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: Cow::Owned(guess),
                mask: correctness,
            });
        }
        None
    }
}

impl Correctness {
    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];

        //Mark things green
        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }
        //Mark things yellow
        let mut used = [false; 5];
        for (i, &c) in c.iter().enumerate() {
            if c == Correctness::Correct {
                used[i] = true;
            }
        }
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                //This case was handled above - marked as green
                continue;
            }

            if answer.chars().enumerate().any(|(i, a)| {
                if a == g && !used[i] {
                    used[i] = true;
                    return true;
                }
                false
            }) {
                c[i] = Correctness::Misplaced;
            }
        }
        c
    }

    pub fn patterns() -> impl Iterator<Item = [Self; 5]> {
        itertools::iproduct!(
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong]
        )
        .map(|(a, b, c, d, e)| [a, b, c, d, e])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correctness {
    //Green
    Correct,
    //Yellow
    Misplaced,
    //Gray
    Wrong,
}

pub struct Guess<'a> {
    pub word: Cow<'a, str>,
    pub mask: [Correctness; 5],
}

impl Guess<'_> {
    pub fn matches(&self, word: &str) -> bool {
        assert_eq!(self.word.len(), 5);
        assert_eq!(word.len(), 5);

        //Check the greens
        let mut used = [false; 5];
        for (i, ((g, &m), w)) in self
            .word
            .bytes()
            .zip(&self.mask)
            .zip(word.bytes())
            .enumerate()
        {
            if m == Correctness::Correct {
                if g != w {
                    return false;
                } else {
                    used[i] = true;
                    continue;
                }
            }
        }

        //This is the confusing part
        for (i, (w, &m)) in word.bytes().zip(&self.mask).enumerate() {
            if m == Correctness::Correct {
                //Must be correct, or we'd have returned in the earlier loop
                continue;
            }

            let mut plausible = true;
            if self
                .word
                .bytes()
                .zip(&self.mask)
                .enumerate()
                .any(|(j, (g, m))| {
                    if g != w {
                        return false;
                    }
                    if used[j] {
                        return false;
                    }
                    // We're looking at an w in word, have found an w in the previous guess.
                    //The color of that previous w will tell us whether this w might be okay.

                    match m {
                        Correctness::Correct => unreachable!(
                            "all correct guesses should have result in return or be used"
                        ),
                        Correctness::Misplaced if j == i => {
                            plausible = false;
                            return true;
                        }

                        Correctness::Misplaced => {
                            used[j] = true;
                            return true;
                        }
                        Correctness::Wrong => {
                            //Early return
                            //dbg!(i, w, j, g, m);
                            plausible = false;
                            return false;
                        }
                    }
                })
                && plausible
            {
                // The character w was yellow in the previous guess.
                assert!(plausible);
            } else if !plausible {
                return false;
            } else {
                //We have no information about character w, so word might still match
            }
        }
        true
    }
}

//String is the next guess it wants to make given its history
pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}

impl Guesser for fn(history: &[Guess]) -> String {
    fn guess(&mut self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl $crate::Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

#[cfg(test)]
macro_rules! mask {
    (C) => {$crate::Correctness::Correct};
    (M) => {$crate::Correctness::Misplaced};
    (W) => {$crate::Correctness::Wrong};
    ($($c:tt)+) => {[
        $(mask!($c)),+
    ]}
}

#[cfg(test)]
mod tests {

    mod guess_matcher {
        use crate::Guess;

        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess {
                    word: $prev.to_string(),
                    mask: mask![$($mask )+]
                }
                .matches($next))
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess {
                    word: $prev.to_string(),
                    mask: mask![$($mask )+]
                }
                .matches($next))
            }
        }

        #[test]
        fn matches() {
            check!("abcde" + [C C C C C] allows "abcde");
            check!("abcdf" + [C C C C C] disallows "abcde");
            check!("abcde" + [W W W W W] allows "fghij");
            check!("abcdf" + [M M M M M] allows "eabcd");
            check!("baaaa" + [W C M W W] disallows "caacc");
        }

        #[test]
        fn debug() {
            check!("baaaa" + [W C M W W] allows "aaccc");
        }

        #[test]
        fn from_chat() {
            //flocular
            check!("aaabb" + [C M W W W] disallows "accaa");
            //ritoban
            check!("abcde" + [W W W W W] disallows "bcdea");
        }
    }
    mod game {
        use crate::{Guess, Wordle};
        #[test]
        fn genius() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "right".to_string() });
            assert_eq!(w.play("right", guesser), Some(1));
        }

        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(2));
        }

        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(3));
        }
        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(4));
        }

        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(5));
        }

        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(6));
        }

        #[test]
        fn oops() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "wrong".to_string() });
            assert_eq!(w.play("right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask![C C C C C]);
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcde", "fghij"), mask![W W W W W]);
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "eabcd"), mask![M M M M M]);
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask![C C W W W]);
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask![W W M M W]);
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask![W C M W W]);
        }

        #[test]
        fn from_chat_1() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask![C M W W W]);
        }

        #[test]
        fn from_chat_2() {
            assert_eq!(Correctness::compute("baccc", "aaddd"), mask![W C W W W]);
        }

        #[test]
        fn from_chat_3() {
            assert_eq!(Correctness::compute("abcde", "aacde"), mask![C W C C C]);
        }
    }
}
