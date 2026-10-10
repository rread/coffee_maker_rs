//! World state, the verbs, and the read–parse–act loop.

use std::io::{self, BufRead, Write};
use std::time::Duration;

use crate::Pacer;
use crate::parser::{Binding, Command, Dictionary, TooManyWords};
use crate::text::wrap;
use crate::thing::{Direction, Room, Thing, Verb, Word};

const LINE_PAUSE: Duration = Duration::from_millis(100);
const COMMAND_PAUSE: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Location {
    Nowhere,
    Carried,
    In(Room),
}

/// Everything that changes during play.
struct World {
    /// Where the avatar is.
    room: Room,
    /// Where every other thing is (the avatar's own entry is unused).
    locations: [Location; Thing::COUNT],
    visited: [bool; Room::COUNT],
    /// Once the kettle sits in its stand it can't be taken again.
    kettle_in_stand: bool,
    /// Making coffee opens the stairs from the kitchen to the sun room.
    coffee_made: bool,
    words: Dictionary,
    done: bool,
}

impl World {
    fn new() -> Self {
        let mut locations = [Location::Nowhere; Thing::COUNT];
        locations[Thing::Kettle as usize] = Location::In(Room::Dining);
        locations[Thing::DirtyFunnel as usize] = Location::In(Room::Library);
        locations[Thing::DirtyMug as usize] = Location::In(Room::Lounge);
        locations[Thing::Note as usize] = Location::In(Room::Kitchen);
        locations[Thing::SackOfFoodstuffs as usize] = Location::In(Room::Basement);
        let mut visited = [false; Room::COUNT];
        visited[Room::Kitchen as usize] = true;
        World {
            room: Room::Kitchen,
            locations,
            visited,
            kettle_in_stand: false,
            coffee_made: false,
            words: Dictionary::new(),
            done: false,
        }
    }

    fn location(&self, t: Thing) -> Location {
        match t {
            Thing::Avatar => Location::In(self.room),
            _ => self.locations[t as usize],
        }
    }

    fn put(&mut self, t: Thing, loc: Location) {
        self.locations[t as usize] = loc;
    }

    fn carries(&self, t: Thing) -> bool {
        self.location(t) == Location::Carried
    }

    fn is_here(&self, t: Thing) -> bool {
        self.location(t) == Location::In(self.room)
    }

    /// Carried, or in the same room as the avatar.
    fn can_reach(&self, t: Thing) -> bool {
        self.carries(t) || self.is_here(t)
    }

    fn can_take(&self, t: Thing) -> bool {
        t.is_takeable() && !(t == Thing::Kettle && self.kettle_in_stand)
    }

    fn exit(&self, from: Room, dir: Direction) -> Option<Room> {
        use Direction::*;
        match (from, dir) {
            (Room::Kitchen, North) => Some(Room::Dining),
            (Room::Kitchen, East) => Some(Room::Library),
            (Room::Kitchen, South | West) => Some(Room::Lounge),
            (Room::Kitchen, Down) => Some(Room::Basement),
            (Room::Kitchen, Up) if self.coffee_made => Some(Room::Sun),
            (Room::Dining, South) => Some(Room::Kitchen),
            (Room::Library, West) => Some(Room::Kitchen),
            (Room::Lounge, North | East) => Some(Room::Kitchen),
            (Room::Basement, Up) => Some(Room::Kitchen),
            _ => None,
        }
    }
}

pub struct Game<R, W, P> {
    input: R,
    out: W,
    pacer: P,
    world: World,
}

impl<R: BufRead, W: Write, P: Pacer> Game<R, W, P> {
    pub fn new(input: R, out: W, pacer: P) -> Self {
        Game {
            input,
            out,
            pacer,
            world: World::new(),
        }
    }

    /// Play until the player wins, types `@quit`, or input ends.
    pub fn run(&mut self) -> io::Result<()> {
        writeln!(self.out, "Begin")?;
        writeln!(
            self.out,
            "There are {} words in the dictionary.",
            self.world.words.len()
        )?;
        writeln!(self.out, "***")?;
        self.look_here()?;
        while !self.world.done {
            writeln!(self.out, "---")?;
            write!(self.out, "? ")?;
            self.out.flush()?;
            let Some(line) = self.read_line()? else { break };
            if line == "@quit" {
                break;
            }
            match self.world.words.parse(&line) {
                Ok(cmd) => {
                    self.pacer.pause(COMMAND_PAUSE);
                    self.do_command(&cmd)?;
                }
                Err(TooManyWords) => {
                    writeln!(self.out, "Nope!  I'm a 2-word parser.  Try again.")?;
                }
            }
        }
        writeln!(self.out, "End")?;
        self.out.flush()
    }

    fn read_line(&mut self) -> io::Result<Option<String>> {
        let mut buf = Vec::new();
        if self.input.read_until(b'\n', &mut buf)? == 0 {
            return Ok(None);
        }
        if buf.last() == Some(&b'\n') {
            buf.pop();
        }
        Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
    }

    /// Print a word-wrapped paragraph, pausing after each line.
    fn paragraph(&mut self, s: &str) -> io::Result<()> {
        if s.is_empty() {
            return writeln!(self.out);
        }
        for line in wrap(s) {
            self.out.write_all(line)?;
            self.out.write_all(b"\n")?;
            self.out.flush()?;
            self.pacer.pause(LINE_PAUSE);
        }
        Ok(())
    }

    fn do_command(&mut self, c: &Command) -> io::Result<()> {
        let verb = match c.verb {
            Binding::Unsupplied => return writeln!(self.out, "Hum dee dum dum do hum."),
            Binding::Known(Word::Verb(v)) => v,
            _ => return self.not_a_verb(c),
        };
        match verb {
            Verb::Go(dir) => self.go(dir),
            Verb::Walk => self.walk(c),
            Verb::Look => self.look(c),
            Verb::Drink => writeln!(self.out, "FIXME Drink_Action"),
            Verb::Drop => self.drop(c),
            Verb::Inventory => self.inventory(),
            Verb::Make => self.make(c),
            Verb::Read => self.read(c),
            Verb::Search => self.search(c),
            Verb::Take => self.take(c),
            Verb::Wash => self.wash(c),
            Verb::Wipe => self.paragraph(&format!("FIXME You wipe {}.", c.object_word)),
            Verb::Lock | Verb::Unlock | Verb::Use => self.not_a_verb(c),
        }
    }

    fn not_a_verb(&mut self, c: &Command) -> io::Result<()> {
        writeln!(self.out, "\"{}\" is not a verb!", c.verb_word)
    }

    fn look_here(&mut self) -> io::Result<()> {
        let room = self.world.room;
        self.paragraph(room.desc())?;
        let names: Vec<_> = Thing::ALL
            .into_iter()
            .filter(|&t| t != Thing::Avatar && self.world.is_here(t))
            .map(Thing::name)
            .collect();
        if !names.is_empty() {
            write!(self.out, "You see {}.", names.join(", "))?;
        }
        writeln!(self.out)
    }

    fn go(&mut self, dir: Direction) -> io::Result<()> {
        match self.world.exit(self.world.room, dir) {
            None => writeln!(self.out, "You can't go that way.")?,
            Some(target) => {
                self.world.room = target;
                let visited = &mut self.world.visited[target as usize];
                if *visited {
                    writeln!(self.out, "You are {}.", target.name())?;
                } else {
                    *visited = true;
                    self.look_here()?;
                }
            }
        }
        if dir == Direction::Up && self.world.room == Room::Sun {
            self.world.done = true;
        }
        Ok(())
    }

    fn walk(&mut self, c: &Command) -> io::Result<()> {
        match c.object {
            Binding::Known(Word::Verb(Verb::Go(dir))) => self.go(dir),
            // No newline, as in the original.
            _ => write!(self.out, "\"{}\" isn't a direction.", c.object_word),
        }
    }

    fn look(&mut self, c: &Command) -> io::Result<()> {
        match c.object {
            Binding::Unsupplied | Binding::Known(Word::Here) => self.look_here(),
            obj => match obj.thing().filter(|&t| self.world.can_reach(t)) {
                Some(t) => self.paragraph(t.desc()),
                None => writeln!(self.out, "I don't see any {} here.", c.object_word),
            },
        }
    }

    fn read(&mut self, c: &Command) -> io::Result<()> {
        match c.object {
            Binding::Unsupplied => writeln!(self.out, "Read what?"),
            Binding::Unknown => writeln!(self.out, "What is a \"{}\"?", c.object_word),
            obj => match obj.thing().filter(|&t| self.world.can_reach(t)) {
                Some(t) => self.paragraph(t.desc()),
                None => writeln!(self.out, "I don't see any {} here.", c.object_word),
            },
        }
    }

    fn take(&mut self, c: &Command) -> io::Result<()> {
        let w = &c.object_word;
        match c.object {
            Binding::Unsupplied => writeln!(self.out, "Take what?"),
            Binding::Unknown => writeln!(self.out, "What is a {w}?"),
            obj => match obj.thing().filter(|&t| self.world.is_here(t)) {
                None => writeln!(self.out, "I don't see any {w} here."),
                Some(t) if !self.world.can_take(t) => writeln!(self.out, "You can't take that."),
                Some(t) => {
                    self.world.put(t, Location::Carried);
                    writeln!(self.out, "{w}: Taken.")
                }
            },
        }
    }

    fn drop(&mut self, c: &Command) -> io::Result<()> {
        let w = &c.object_word;
        match c.object {
            Binding::Unsupplied => writeln!(self.out, "Drop what?"),
            Binding::Unknown => writeln!(self.out, "What is a {w}?"),
            obj => match obj.thing().filter(|&t| self.world.carries(t)) {
                None => writeln!(self.out, "You aren't carrying any {w}."),
                Some(Thing::MugOfCoffee) => writeln!(self.out, "It's impossible to drop {w}!"),
                Some(Thing::Kettle) if self.world.room == Room::Kitchen => {
                    self.world.put(Thing::Kettle, Location::In(Room::Kitchen));
                    self.world.kettle_in_stand = true;
                    self.paragraph(concat!(
                        "The electric kettle fits snuggly into its stand &",
                        " is ready to boil water when you're ready to make",
                        " coffee!"
                    ))
                }
                Some(t) => {
                    self.world.put(t, Location::In(self.world.room));
                    writeln!(self.out, "{w}: Dropped.")
                }
            },
        }
    }

    fn inventory(&mut self) -> io::Result<()> {
        let carried: Vec<_> = Thing::ALL
            .into_iter()
            .filter(|&t| self.world.carries(t))
            .collect();
        write!(self.out, "You are carrying {} items.", carried.len())?;
        if carried.is_empty() {
            return writeln!(self.out);
        }
        writeln!(self.out, "  They are...")?;
        for t in carried {
            writeln!(self.out, "* {}", t.name())?;
        }
        Ok(())
    }

    fn search(&mut self, c: &Command) -> io::Result<()> {
        let w = &c.object_word;
        match c.object {
            Binding::Unsupplied => writeln!(self.out, "Search what?"),
            Binding::Unknown => writeln!(self.out, "What is a {w}?"),
            obj => match obj.thing().filter(|&t| self.world.can_reach(t)) {
                None => self.paragraph(&format!(
                    "You aren't carrying any {w}, & there isn't one here in the room."
                )),
                Some(Thing::SackOfFoodstuffs) => {
                    let sack = self.world.location(Thing::SackOfFoodstuffs);
                    self.world.put(Thing::GroundCoffee, sack);
                    self.world.put(Thing::SackOfFoodstuffs, Location::Nowhere);
                    self.paragraph(&format!(
                        "Searching {} reveals {}!",
                        Thing::SackOfFoodstuffs.name(),
                        Thing::GroundCoffee.name()
                    ))
                }
                Some(_) => self.paragraph(&format!("Searching {w} doesn't reveal anything.")),
            },
        }
    }

    fn wash(&mut self, c: &Command) -> io::Result<()> {
        if c.object == Binding::Unknown {
            return self.paragraph(&format!("What is a {}?", c.object_word));
        }
        let Some(t) = c.object.thing().filter(|&t| self.world.carries(t)) else {
            return self.paragraph("You can't wash it if you aren't carrying it.");
        };
        if self.world.room != Room::Kitchen {
            return self.paragraph(concat!(
                "The only place around here for washing anything is",
                " the kitchen."
            ));
        }
        match t {
            Thing::DirtyMug => {
                self.world.put(Thing::CleanMug, Location::Carried);
                self.world.put(Thing::DirtyMug, Location::Nowhere);
                self.world.words.rebind("mug", Thing::CleanMug);
                self.paragraph(concat!(
                    "The formerly dirty mug is now clean enough for a drink.",
                    "  Excellent!"
                ))
            }
            Thing::DirtyFunnel => {
                self.world.put(Thing::CleanFunnel, Location::Carried);
                self.world.put(Thing::DirtyFunnel, Location::Nowhere);
                self.world.words.rebind("funnel", Thing::CleanFunnel);
                self.world.words.rebind("filter", Thing::CleanFunnel);
                self.paragraph(concat!(
                    "The pour-over funnel is now clean & ready for making",
                    " coffee.  Wonderful!"
                ))
            }
            _ => self.paragraph("It's cleaner than it was, always an improvement."),
        }
    }

    fn make(&mut self, c: &Command) -> io::Result<()> {
        let w = &self.world;
        let failure = if c.object.thing() != Some(Thing::GroundCoffee) {
            Some(concat!(
                "You could make coffee if you have all 4 of the items &",
                " you are in the kitchen, but I'm not aware of anything",
                " else you could make in this game.  Sorry."
            ))
        } else if w.room != Room::Kitchen {
            Some("The kitchen is the best place to make coffee.")
        } else if w.location(Thing::Kettle) != Location::In(Room::Kitchen) {
            Some(concat!(
                "You'll need to drop an electric kettle on the electric",
                " kettle stand that's on the counter top over there."
            ))
        } else if !w.can_reach(Thing::CleanMug) {
            Some(concat!(
                "You'll need to hold a clean mug or at least have one",
                " here in the kitchen."
            ))
        } else if !w.can_reach(Thing::CleanFunnel) {
            Some(concat!(
                "You'll need to carry a clean pour-over funnel, or at",
                " least have one here with you in the kitchen.  And",
                " it must be clean, not just a dirty old thing."
            ))
        } else if !w.can_reach(Thing::GroundCoffee) {
            Some(concat!(
                "You'll need to carry some ground coffee or have it",
                " here in the kitchen.  I mean, you can't make coffee",
                " without some ground coffee."
            ))
        } else {
            None
        };
        if let Some(msg) = failure {
            return self.paragraph(msg);
        }

        self.world.put(Thing::MugOfCoffee, Location::Carried);
        for used in [Thing::CleanMug, Thing::CleanFunnel, Thing::GroundCoffee] {
            self.world.put(used, Location::Nowhere);
        }
        self.world.coffee_made = true;
        self.world.words.rebind("coffee", Thing::MugOfCoffee);
        self.paragraph(concat!(
            "You fill the electric kettle with water from the faucet",
            " & start it.  You put the pour-over funnel on your mug &",
            " fill it with some ground coffee.  You wait..."
        ))?;
        self.paragraph(concat!(
            "When the water boils, you pour it into the filter &",
            " wait while it trickles into the mug."
        ))?;
        self.paragraph(concat!(
            "You now have a cup of steaming coffee!  It smells",
            " wonderful!"
        ))?;
        self.paragraph(concat!(
            "A kitty enters from some direction you didn't see,",
            " meows, & runs up some stairs you hadn't noticed",
            " before.  There's a sun room up there, perfect for",
            " sipping coffee with a friend.  Someone is waiting for",
            " you up there."
        ))
    }
}
