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
def_char            0x6,ACK;
def_char            0x7,BEL;
def_char            0x8,BACKSPACE;
def_char            0x9,TAB;
def_char            0xA,LINE_FEED;
def_char            0xB,VERTICAL_TAB;
def_char            0xC,FORM_FEED;
def_char            0xD,CARRIAGE_RETURN;
def_char            0xE,SHIFT_OUT;
def_char            0xF,SHIFT_IN;
def_char            0x10,DATA_LINK_ESCAPE;
def_char            0x11,DEVICE_CONTROL_1;
def_char            0x12,DEVICE_CONTROL_2;
def_char            0x13,DEVICE_CONTROL_3;
def_char            0x14,DEVICE_CONTROL_4;
def_char            0x15,NEGATIVE_ACK;
def_char            0x16,SYNC_IDLE;
def_char            0x17,END_OF_TRANS_BLOCK;
def_char            0x18,CANCEL;
def_char            0x19,END_OF_MEDIUM;
def_char            0x1A,SUBSTITUTE;
def_char            0x1B,ESCAPE;
def_char            0x1C,FILE_SEPARATOR;
def_char            0x1D,GROUP_SEPARATOR;
def_char            0x1E,RECORD_SEPARATOR;
def_char            0x1F,UNIT_SEPARATOR;
def_char            0x20,SPACE;
def_char            0x21,EXCLAMATION_POINT;
def_char            0x22,DOUBLE_QUOTE;
def_char            0x23,POUND_SIGN;
def_char            0x24,DOLLAR_SIGN;
def_char            0x25,PERCENT_SIGN;
def_char            0x26,AMPERSAND;
def_char            0x27,SINGLE_QUOTE;
def_char            0x28,OPEN_PARENTHESIS;
def_char            0x29,CLOSE_PARENTHESIS;
def_char            0x2A,STAR_SIGN;
def_char            0x2B,PLUS_SIGN;
def_char            0x2C,COMMA;
def_char            0x2D,MINUS_SIGN;
def_char            0x2E,PERIOD;
def_char            0x2F,SLASH;
def_char            0x30,ZERO;
def_char            0x31,ONE;
def_char            0x32,TWO;
def_char            0x33,THREE;
def_char            0x34,FOUR;
def_char            0x35,FIVE;
def_char            0x36,SIX;
def_char            0x37,SEVEN;
def_char            0x38,EIGHT;
def_char            0x39,NINE;
def_char            0x3A,COLON;
def_char            0x3B,SEMICOLON;
def_char            0x3C,LESS_THAN_SIGN;
def_char            0x3D,EQUALS_SIGN;
def_char            0x3E,GREATER_THAN_SIGN;
def_char            0x3F,QUESTION_MARK;
def_char            0x40,AT_SIGN;
def_char            0x41,A_UPPERCASE;
def_char            0x42,B_UPPERCASE;
def_char            0x43,C_UPPERCASE;
def_char            0x44,D_UPPERCASE;
def_char            0x45,E_UPPERCASE;
def_char            0x46,F_UPPERCASE;
def_char            0x47,G_UPPERCASE;
def_char            0x48,H_UPPERCASE;
def_char            0x49,I_UPPERCASE;
def_char            0x4A,J_UPPERCASE;
def_char            0x4B,K_UPPERCASE;
def_char            0x4C,L_UPPERCASE;
def_char            0x4D,M_UPPERCASE;
def_char            0x4E,N_UPPERCASE;
def_char            0x4F,O_UPPERCASE;
def_char            0x50,P_UPPERCASE;
def_char            0x51,Q_UPPERCASE;
def_char            0x52,R_UPPERCASE;
def_char            0x53,S_UPPERCASE;
def_char            0x54,T_UPPERCASE;
def_char            0x55,U_UPPERCASE;
def_char            0x56,V_UPPERCASE;
def_char            0x57,W_UPPERCASE;
def_char            0x58,X_UPPERCASE;
def_char            0x59,Y_UPPERCASE;
def_char            0x5A,Z_UPPERCASE;
def_char            0x5B,SQUARE_BRACKET_LEFT;
def_char            0x5C,BACKWARDS_SLASH;
def_char            0x5D,SQUARE_BRACKET_RIGHT;
def_char            0x5E,CARET;
def_char            0x5F,UNDERSCORE;
def_char            0x60,BACK_TICK;
def_char            0x61,A_LOWERCASE;
def_char            0x62,B_LOWERCASE;
def_char            0x63,C_LOWERCASE;
def_char            0x64,D_LOWERCASE;
def_char            0x65,E_LOWERCASE;
def_char            0x66,F_LOWERCASE;
def_char            0x67,G_LOWERCASE;
def_char            0x68,H_LOWERCASE;
def_char            0x69,I_LOWERCASE;
def_char            0x6A,J_LOWERCASE;
def_char            0x6B,K_LOWERCASE;
def_char            0x6C,L_LOWERCASE;
def_char            0x6D,M_LOWERCASE;
def_char            0x6E,N_LOWERCASE;
def_char            0x6F,O_LOWERCASE;
def_char            0x70,P_LOWERCASE;
def_char            0x71,Q_LOWERCASE;
def_char            0x72,R_LOWERCASE;
def_char            0x73,S_LOWERCASE;
def_char            0x74,T_LOWERCASE;
def_char            0x75,U_LOWERCASE;
def_char            0x76,V_LOWERCASE;
def_char            0x77,W_LOWERCASE;
def_char            0x78,X_LOWERCASE;
def_char            0x79,Y_LOWERCASE;
def_char            0x7A,Z_LOWERCASE;
def_char            0x7B,OPEN_CURLY_BRACKET;
def_char            0x7C,PIPE;
def_char            0x7D,CLOSE_CURLY_BRACKET;
def_char            0x7E,TILDE;
def_char            0x7F,DELETE;

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
push_char           H_UPPERCASE,A;
push_char           E_LOWERCASE,A;
push_char           L_LOWERCASE,A;
push_char           L_LOWERCASE,A;
push_char           O_LOWERCASE,A;
push_val            0x2C,A;
push_val            0x20,A;
push_char           W_UPPERCASE,A;
push_char           O_LOWERCASE,A;
push_char           R_LOWERCASE,A;
push_char           L_LOWERCASE,A;
push_char           D_LOWERCASE,A;
push_val            0x21,A;
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