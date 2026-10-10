//! Everything in the world: rooms, the things in them, and the verbs and
//! words the parser understands.

/// The rooms of the house.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Room {
    Kitchen,
    Dining,
    Library,
    Lounge,
    Basement,
    Sun,
}

impl Room {
    pub const COUNT: usize = 6;

    /// Used in "You are <name>." when returning to a room.
    pub fn name(self) -> &'static str {
        match self {
            Room::Kitchen => "in the kitchen",
            Room::Dining => "the dining room",
            Room::Lounge => "in the lounge",
            Room::Basement => "in the basement",
            Room::Library | Room::Sun => "<<noname>>",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Room::Kitchen => concat!(
                "You are in a huge kitchen with red-brick walls.  It has ovens",
                " embedded into the walls, three on each side.  There's a table",
                " near the center of the room.  On the table are a stand for an",
                " electric kettle & a note next to it.  There are open doorways",
                " to each of the 4 main compass directions (north, east, south",
                " west)."
            ),
            Room::Dining => concat!(
                "You are in the dining room.  There's a wooden table in the center",
                " of the room.  The kitchen is to the south."
            ),
            Room::Library => concat!(
                "You are in the library.  Tall bookcases line the walls.  There's a",
                " ladder on casters & attached to a rail that runs around the room",
                " so that people (including you) can reach even the top shelves. ",
                " Good thing, that, as those shelves are too high for even the",
                " tallest person you've ever met to reach.  The kitchen is to the",
                " west."
            ),
            Room::Lounge => concat!(
                "You are in the lounge.  A large window on the west lets in sunlight",
                " on bad days, twilight or moonlight when the weather is good.  It's",
                " furnished with comfy chairs & a comfy couch, perfect for people to",
                " take a seat with their coffee to talk or read quietly."
            ),
            Room::Basement => concat!(
                "You are in the basement.  It's dark, but there are no grues.  Such",
                " luck!  The room is large & unfinished.  The floor is cinderblock",
                " with a thick coat of dry dust.  The walls are brick & coated with",
                " nearly as much dust.  Several thick columns of brick prevent",
                " the ceiling from collapsing."
            ),
            Room::Sun => concat!(
                "You walk up the stairs & into a comfortably welcoming room, perfect",
                " for sipping coffee with the kitty & a friend.  Congratulations,",
                " you've completed the game!"
            ),
        }
    }
}

/// Things that can be referred to and (mostly) have a location.
///
/// The declaration order matters: rooms and the inventory list their
/// contents in this order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Thing {
    Avatar,
    CleanFunnel,
    DirtyMug,
    GroundCoffee,
    Kettle,
    CleanMug,
    DirtyFunnel,
    Kitty,
    MugOfCoffee,
    Note,
    SackOfFoodstuffs,
}

impl Thing {
    pub const ALL: [Thing; 11] = [
        Thing::Avatar,
        Thing::CleanFunnel,
        Thing::DirtyMug,
        Thing::GroundCoffee,
        Thing::Kettle,
        Thing::CleanMug,
        Thing::DirtyFunnel,
        Thing::Kitty,
        Thing::MugOfCoffee,
        Thing::Note,
        Thing::SackOfFoodstuffs,
    ];
    pub const COUNT: usize = Self::ALL.len();

    pub fn name(self) -> &'static str {
        match self {
            Thing::Avatar => "you",
            Thing::CleanFunnel => "a pour-over funnel that's ready to make coffee",
            Thing::CleanMug => "a large, clean coffee mug",
            Thing::DirtyFunnel => "a pour-over funnel that needs cleaning",
            Thing::GroundCoffee => "some ground coffee",
            Thing::Kettle => "an electric kettle",
            Thing::DirtyMug => "a large, dirty coffee mug",
            Thing::Note => "a hand-written note",
            Thing::SackOfFoodstuffs => "a sack of drygoods from the grocery store",
            Thing::Kitty | Thing::MugOfCoffee => "<<noname>>",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Thing::Avatar => "a respectable person",
            Thing::CleanMug => concat!(
                "It's a coffee mug.  It's clean enough to drink from if there were",
                " coffee in it."
            ),
            Thing::Kettle => concat!(
                "It's an electric kettle.  The barrel is glass.  Its lid is hinged.",
                "  It has a base with bumps & slots, looks like they fit into a",
                " custom stand."
            ),
            Thing::Note => concat!(
                "In an admirably controlled hand, someone has written",
                " \"To make coffee, you will need the electric kettle, a mug,",
                " a pour-over funnel with built-in filter, & of course some ground",
                " coffee.\""
            ),
            Thing::SackOfFoodstuffs => concat!(
                "It's a paper sack, like from a grocery story.  It's stuffed full",
                " of drygoods (stuff that you can put on a shelf for weeks or",
                " longer without it going bad).  You feel a temptation to SEARCH",
                " this sack."
            ),
            _ => "<<no desc>>",
        }
    }

    pub fn is_takeable(self) -> bool {
        matches!(
            self,
            Thing::CleanMug
                | Thing::DirtyMug
                | Thing::CleanFunnel
                | Thing::DirtyFunnel
                | Thing::GroundCoffee
                | Thing::Kettle
                | Thing::SackOfFoodstuffs
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verb {
    Go(Direction),
    Walk,
    Look,
    Drink,
    Drop,
    Inventory,
    Lock,
    Make,
    Read,
    Search,
    Take,
    Unlock,
    Use,
    Wash,
    Wipe,
}

/// What a dictionary word refers to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Word {
    Thing(Thing),
    Verb(Verb),
    /// "here": the avatar's current room.
    Here,
}
