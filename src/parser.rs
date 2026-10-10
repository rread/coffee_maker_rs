//! The 2-word parser: split a line into a verb and an object, then bind
//! each word through the dictionary.

use std::collections::HashMap;

use crate::thing::{Direction, Thing, Verb, Word};

/// The result of looking a word up in the dictionary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Binding {
    /// The player didn't type this word.
    Unsupplied,
    /// The player typed a word the dictionary doesn't know.
    Unknown,
    Known(Word),
}

impl Binding {
    pub fn thing(self) -> Option<Thing> {
        match self {
            Binding::Known(Word::Thing(t)) => Some(t),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Command {
    pub verb_word: String,
    pub verb: Binding,
    pub object_word: String,
    pub object: Binding,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TooManyWords;

/// Split a line into at most two space-separated words. Only `' '` counts
/// as a separator; a missing word is returned as `""`.
pub fn split_into_words(line: &str) -> Result<(&str, &str), TooManyWords> {
    let mut words = line.split(' ').filter(|w| !w.is_empty());
    let verb = words.next().unwrap_or("");
    let object = words.next().unwrap_or("");
    match words.next() {
        Some(_) => Err(TooManyWords),
        None => Ok((verb, object)),
    }
}

pub struct Dictionary {
    words: HashMap<&'static str, Word>,
}

impl Dictionary {
    pub fn new() -> Self {
        use Direction::*;
        use Thing::*;
        let entries = [
            ("bag", Word::Thing(SackOfFoodstuffs)),
            ("cat", Word::Thing(Kitty)),
            ("coffee", Word::Thing(GroundCoffee)),
            ("d", Word::Verb(Verb::Go(Down))),
            ("down", Word::Verb(Verb::Go(Down))),
            ("drink", Word::Verb(Verb::Drink)),
            ("drop", Word::Verb(Verb::Drop)),
            ("drygoods", Word::Thing(SackOfFoodstuffs)),
            ("e", Word::Verb(Verb::Go(East))),
            ("east", Word::Verb(Verb::Go(East))),
            ("electric", Word::Thing(Kettle)),
            ("exam", Word::Verb(Verb::Look)),
            ("examine", Word::Verb(Verb::Look)),
            ("filter", Word::Thing(DirtyFunnel)),
            ("food", Word::Thing(SackOfFoodstuffs)),
            ("foodstuffs", Word::Thing(SackOfFoodstuffs)),
            ("funnel", Word::Thing(DirtyFunnel)),
            ("get", Word::Verb(Verb::Take)),
            ("go", Word::Verb(Verb::Walk)),
            ("groceries", Word::Thing(SackOfFoodstuffs)),
            ("grounds", Word::Thing(GroundCoffee)),
            ("here", Word::Here),
            ("i", Word::Verb(Verb::Inventory)),
            ("invent", Word::Verb(Verb::Inventory)),
            ("inventory", Word::Verb(Verb::Inventory)),
            ("kettle", Word::Thing(Kettle)),
            ("kitten", Word::Thing(Kitty)),
            ("kitty", Word::Thing(Kitty)),
            ("l", Word::Verb(Verb::Look)),
            ("lock", Word::Verb(Verb::Lock)),
            ("look", Word::Verb(Verb::Look)),
            ("make", Word::Verb(Verb::Make)),
            ("me", Word::Thing(Avatar)),
            ("mug", Word::Thing(DirtyMug)),
            ("n", Word::Verb(Verb::Go(North))),
            ("north", Word::Verb(Verb::Go(North))),
            ("note", Word::Thing(Note)),
            ("read", Word::Verb(Verb::Read)),
            ("rinse", Word::Verb(Verb::Wash)),
            ("s", Word::Verb(Verb::Go(South))),
            ("sack", Word::Thing(SackOfFoodstuffs)),
            ("search", Word::Verb(Verb::Search)),
            ("self", Word::Thing(Avatar)),
            ("south", Word::Verb(Verb::Go(South))),
            ("take", Word::Verb(Verb::Take)),
            ("u", Word::Verb(Verb::Go(Up))),
            ("unlock", Word::Verb(Verb::Unlock)),
            ("up", Word::Verb(Verb::Go(Up))),
            ("use", Word::Verb(Verb::Use)),
            ("w", Word::Verb(Verb::Go(West))),
            ("walk", Word::Verb(Verb::Walk)),
            ("wash", Word::Verb(Verb::Wash)),
            ("wipe", Word::Verb(Verb::Wipe)),
            ("west", Word::Verb(Verb::Go(West))),
        ];
        Dictionary {
            words: HashMap::from(entries),
        }
    }

    /// Number of entries, counting the empty word the way the Ada
    /// dictionary did (it held an explicit `"" => UNSUPPLIED` entry).
    pub fn len(&self) -> usize {
        self.words.len() + 1
    }

    pub fn bind(&self, word: &str) -> Binding {
        if word.is_empty() {
            return Binding::Unsupplied;
        }
        self.words
            .get(word)
            .map_or(Binding::Unknown, |&w| Binding::Known(w))
    }

    /// Point an existing word at a different thing.
    pub fn rebind(&mut self, word: &str, thing: Thing) {
        let entry = self
            .words
            .get_mut(word)
            .unwrap_or_else(|| panic!("rebind: {word:?} is not in the dictionary"));
        *entry = Word::Thing(thing);
    }

    pub fn parse(&self, line: &str) -> Result<Command, TooManyWords> {
        let (verb, object) = split_into_words(line)?;
        Ok(Command {
            verb_word: verb.to_owned(),
            verb: self.bind(verb),
            object_word: object.to_owned(),
            object: self.bind(object),
        })
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_examples_from_ada_source() {
        assert_eq!(split_into_words(""), Ok(("", "")));
        assert_eq!(split_into_words("a"), Ok(("a", "")));
        assert_eq!(split_into_words("abc"), Ok(("abc", "")));
        assert_eq!(split_into_words(" a "), Ok(("a", "")));
        assert_eq!(split_into_words("      "), Ok(("", "")));
        assert_eq!(split_into_words("a b"), Ok(("a", "b")));
        assert_eq!(split_into_words("word1 word2"), Ok(("word1", "word2")));
        assert_eq!(split_into_words("word1   word2"), Ok(("word1", "word2")));
        assert_eq!(split_into_words(" word1   word2 "), Ok(("word1", "word2")));
        assert_eq!(split_into_words("one two three"), Err(TooManyWords));
    }

    #[test]
    fn only_spaces_separate_words() {
        assert_eq!(split_into_words("look\tnote"), Ok(("look\tnote", "")));
        assert_eq!(split_into_words("look\r"), Ok(("look\r", "")));
    }

    #[test]
    fn dictionary_has_55_words() {
        assert_eq!(Dictionary::new().len(), 55);
    }

    #[test]
    fn binding() {
        let mut d = Dictionary::new();
        assert_eq!(d.bind(""), Binding::Unsupplied);
        assert_eq!(d.bind("Look"), Binding::Unknown);
        assert_eq!(d.bind("mug"), Binding::Known(Word::Thing(Thing::DirtyMug)));
        d.rebind("mug", Thing::CleanMug);
        assert_eq!(d.bind("mug"), Binding::Known(Word::Thing(Thing::CleanMug)));
    }
}
