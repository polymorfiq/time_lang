use std::io::{prelude::*, BufReader};

static PROGRAM: &str = r#"
# --- Overview ---
# This is an Assembly Language for a Virtual Machine, that deals with *streams* of *characters* and contextual *time markers*.
# *Characters* are defined via an *Alphabet* - A finite number of bits, a subset of which are valid characters in the alphabet.
# *Moments* are defined via a *Clock* - A finite number of bits may represent a clock's moments.
# *Streams* are (potentially infinite) sources of information that correspond to a given *Alphabet* and *Clock* - They are stateful and once a character or time marker is read, it is forever removed from the stream.

# This script first defines an Alphabet, a Clock, then a set of programs.
# ---

defalphabet ASCII;

# Defines the maximum number of bits a 'character' (atom of data) might take up
set_char_type   u8;

# Defines the 'characters' that can move through a stream
def_char            0x0,NULL_BYTE;
def_char            0x1,START_OF_HEADING;
def_char            0x2,START_OF_TEXT;
def_char            0x3,END_OF_TEXT;
def_char            0x4,END_OF_TRANSMITION;
def_char            0x5,INQUIRY;
...

defclock CounterClock;

# Defines the maximum number of bits that a moment of time might take up
set_moment_type      u32;

# Defines what kind of thing the clock represents, could also be:
    UNIX_TIMESTAMP
    NATURAL_MILLISECONDS
    NATURAL_SECONDS
    NATURAL_MINUTES
    NATURAL_HOURS
    ...
set_clock_repr      QUANTITY;


# --- Programs ---
# Quick explanation of functions:
# reg_gateway       NAME,ALPHABET,CLOCK,BUF     - Register an input stream (Input of program) with BUF buffer size
# reg_exit          NAME,ALPHABET,CLOCK,BUF     - Register an exit stream (Output of program) with BUF buffer size
# start_moment      INITIAL_MOMENT,EXIT         - Defines the "initial" moment that your exit clock will start at
# push_char         CHAR,EXIT                   - Push a character onto the exit stream - can either directly be a character from the related alphabet or a hexadecimal representation of bits.
# push_moment       INCREMENT_BY,EXIT           - Push a time marker onto the exit stream, representing INCREMENTED_BY moments passing
# label             LABEL;                      - A nice label to make it easier to define jumps
# jlt               LABEL,TIME,TIME             - Jumps to a given label, if A is earlier than B - Can only jump *forward* in the program
# jgt               LABEL,TIME,TIME             - Jumps to a given label, if A is later than B - Can only jump *forward* in the program
# forward_duration  GATEWAY,EXIT                - Pops characters off of GATEWAY until it hits the next duration, while PUSHing each of those characters to EXIT
# connect           PROGRAM(GATEWAY...),NAME    - Forwards GATEWAYs to PROGRAM. Exits of the program can be pulled from NAME
# reg_exit_gateway  NAME(EXIT),NAME             - Registers a new Gateway, from the Exit of the connected program

defprogram hello_world;
# Outputs "Hello, World!" in ASCII, within a single moment of time

# Exits: Output stream for the program
reg_exit            A,ASCII,CounterClock,0x50;

# All streams have clocks. What moment does this one start at?
start_moment        0,A;

# A
push_moment         1,A;
push_char           H,A;
push_char           E,A;
push_char           L,A;
push_char           L,A;
push_char           O,A;
push_char           0x2C,A;
push_char           0x20,A;
push_char           W,A;
push_char           O,A;
push_char           R,A;
push_char           L,A;
push_char           D,A;
push_char           !,A;
push_moment         1,A;

defprogram sync2;
# Ensures that two streams are in sync with each other, so that no time duration is missed.

# Example:
#  Gateway A: |2 B |3 D |4
#  Gateway B: |1 A |2 C |5 E
#  Exit C:    |1 |2 B |3 D |4 |5
#  Exit D:    |1 A |2 B |3 D |4 |5 E

reg_gateway         A,ASCII,CounterClock,0x50;
reg_gateway         B,ASCII,CounterClock,0x50;
reg_exit            C,ASCII,CounterClock,0x50;
reg_exit            D,ASCII,CounterClock,0x50;

label main;
jlt                 a_earlier,Time(A),Time(B);
jgt                 a_later,Time(A),Time(B);
forward_duration    A,C;
push_moment         Time(A),C;
forward_duration    B,D;
push_moment         Time(B),D;

label a_earlier;
push_moment         Time(A),D;
forward_duration    A,C;
push_moment         Time(A),C;

label a_later;
push_moment         Time(B),C;
forward_duration    B,D;
push_moment         Time(B),D;

defprogram zip2;
# Interleaves two streams of data - if both occurred in the same moment, the first stream's data comes first.

# Example:
# Gateway A:    1| A 3| C 4| E
# Gateway B:    1| B 3| D
# Exit C:       1| AB 3| CD 4| E

reg_gateway         A,ASCII,CounterClock,0x50;
reg_gateway         B,ASCII,CounterClock,0x50;
reg_exit            E,ASCII,CounterClock,0x50;

connect             sync2(A|B),SYNCED;
reg_exit_gateway    SYNCED(C),C;
reg_exit_gateway    SYNCED(D),D;

label main;
forward_duration    C,E;
forward_duration    D,E;
push_moment         Time(C),E;
"#;

mod parser;
use parser::Parser;

fn main() {
    let mut parser = Parser::new("program");
    let reader = BufReader::new(PROGRAM.as_bytes());

    for line in reader.lines() {
        if line.is_ok() {
            parser.parse_line(line.unwrap());
        }
    }

    match parser.generate() {
        Ok(source) => {
            println!("{}", source);
        }

        Err(err) => {
            panic!("Parsing Error:\n{}", err);
        }
    }
}