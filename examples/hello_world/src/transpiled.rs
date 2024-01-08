use core::default::Default;
use core::fmt::Debug;

#[derive(Debug)]
pub enum AlphabetError<CharRep: Debug> {
    UnknownCharacter(CharRep),
    UnexpectedError(&'static str),
    NameNotFound(),
}
pub trait AlphabetLike {
    type CharRep: Copy + Clone + Debug;
    type CharEnum: Copy + Clone + Debug;
    fn char_with_name(rep: &str) -> Result<Self::CharEnum, AlphabetError<&str>>;
    fn to_char(rep: Self::CharRep) -> Result<Self::CharEnum, AlphabetError<Self::CharRep>>;
    fn to_val(rep: Self::CharEnum) -> Self::CharRep;
}

pub enum ClockMoment<MomentRep> {
    UnixSeconds(MomentRep),
    UnixMilliseconds(MomentRep),
    Quantity(MomentRep),
}
pub trait ClockLike {
    type MomentRep: Copy + Clone + Debug;
    fn represents(&self) -> &str;
    fn to_moment(rep: Self::MomentRep) -> ClockMoment<Self::MomentRep>;
}
pub trait AddableClockLike<MomentRep: core::ops::Add<Output = MomentRep>> {
    fn add(moment: ClockMoment<MomentRep>, rep: MomentRep) -> ClockMoment<MomentRep> {
        match moment {
            ClockMoment::Quantity(orig_rep) => ClockMoment::Quantity(orig_rep + rep),
            ClockMoment::UnixMilliseconds(orig_rep) => {
                ClockMoment::UnixMilliseconds(orig_rep + rep)
            }
            ClockMoment::UnixSeconds(orig_rep) => ClockMoment::UnixSeconds(orig_rep + rep),
        }
    }
}

#[derive(Debug)]
pub enum ExitError {
    BufferFull,
}
pub trait ExitLike<Alphabet: AlphabetLike, Clock: ClockLike> {
    type InternalItem;
    type Item;
    fn set_initial_moment(&mut self, monent: Clock::MomentRep);
    fn accepting_pushes(&mut self) -> bool;
    fn push(&mut self, chr: Alphabet::CharEnum) -> Result<(), ExitError>;
    fn push_moment(&mut self, moment: Clock::MomentRep) -> Result<(), ExitError>;
    fn push_with_name(&mut self, chr_name: &str) -> Result<(), ExitError> {
        self.push(
            Alphabet::char_with_name(chr_name)
                .unwrap_or_else(|_| panic!("Unknown char name: {}", chr_name)),
        )
    }
}
pub trait GatewayLike<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize> {
    type InternalItem;
    type Item;
    fn pop(&mut self) -> Self::Item;
    fn forward_duration<Exit: ExitLike<Alphabet, Clock>>(
        &mut self,
        exit: &mut Exit,
    ) -> Result<(), ExitError>;
    fn current_moment(&self) -> Option<Clock::MomentRep>;
    fn is_empty(&self) -> bool;
    fn next_is_character(&self) -> bool;
    fn next_is_moment(&self) -> bool;
}
#[derive(Copy, Clone, Debug)]
pub enum StreamItem<CharacterRep, Moment> {
    Empty,
    Character(CharacterRep),
    Moment(Moment),
}
impl<CharacterRep, Moment> Default for StreamItem<CharacterRep, Moment> {
    fn default() -> Self {
        Self::Empty
    }
}
pub struct Stream<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize> {
    buffer: [StreamItem<Alphabet::CharRep, Clock::MomentRep>; BUFFER_SIZE],
    idx: usize,
    buffered_total: usize,
    buffered_moments: usize,
    buffered_characters: usize,
    last_seen_moment: Option<Clock::MomentRep>,
}
impl<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize>
    Stream<Alphabet, Clock, BUFFER_SIZE>
{
    pub const fn new() -> Self {
        Self {
            buffer: [StreamItem::Empty; BUFFER_SIZE],
            idx: 0,
            buffered_total: 0,
            buffered_moments: 0,
            buffered_characters: 0,
            last_seen_moment: None,
        }
    }
    fn inc_index(&mut self) {
        self.idx = (self.idx + 1) % BUFFER_SIZE;
    }
}
impl<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize> ExitLike<Alphabet, Clock>
    for Stream<Alphabet, Clock, BUFFER_SIZE>
{
    type InternalItem = StreamItem<Alphabet::CharRep, Clock::MomentRep>;
    type Item = StreamItem<Alphabet::CharEnum, Clock::MomentRep>;
    fn set_initial_moment(&mut self, moment: Clock::MomentRep) {
        self.last_seen_moment = Some(moment);
    }
    fn accepting_pushes(&mut self) -> bool {
        self.buffered_total < BUFFER_SIZE
    }
    fn push(&mut self, chr: Alphabet::CharEnum) -> Result<(), ExitError> {
        if self.accepting_pushes() {
            self.buffer[self.idx] = Self::InternalItem::Character(Alphabet::to_val(chr));
            self.buffered_characters += 1;
            self.buffered_total += 1;
            self.inc_index();
            Ok(())
        } else {
            Err(ExitError::BufferFull)
        }
    }
    fn push_moment(&mut self, moment: Clock::MomentRep) -> Result<(), ExitError> {
        if self.accepting_pushes() {
            self.buffer[self.idx] = Self::InternalItem::Moment(moment);
            self.buffered_moments += 1;
            self.buffered_total += 1;
            self.inc_index();
            Ok(())
        } else {
            Err(ExitError::BufferFull)
        }
    }
}
impl<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize>
    GatewayLike<Alphabet, Clock, BUFFER_SIZE> for Stream<Alphabet, Clock, BUFFER_SIZE>
{
    type InternalItem = StreamItem<Alphabet::CharRep, Clock::MomentRep>;
    type Item = StreamItem<Alphabet::CharEnum, Clock::MomentRep>;
    fn pop(&mut self) -> Self::Item {
        let last = core::mem::take(&mut self.buffer[self.idx]);
        self.inc_index();
        match last {
            Self::InternalItem::Character(chr) => {
                self.buffered_characters -= 1;
                self.buffered_total -= 1;
                Self::Item::Character(Alphabet::to_char(chr).unwrap_or_else(|err| {
                    panic!("Unexpected character received in stream: {:?}", err);
                }))
            }
            Self::InternalItem::Moment(moment) => {
                self.buffered_moments -= 1;
                self.buffered_total -= 1;
                self.last_seen_moment = Some(moment);
                Self::Item::Moment(moment)
            }
            Self::InternalItem::Empty => Self::Item::Empty,
        }
    }
    fn forward_duration<Exit: ExitLike<Alphabet, Clock>>(
        &mut self,
        exit: &mut Exit,
    ) -> Result<(), ExitError> {
        while self.next_is_character() {
            match self.pop() {
                Self::Item::Character(chr) => {
                    let result = exit.push(chr);
                    match result {
                        Ok(_) => (),
                        Err(err) => return Err(err),
                    }
                }
                item => panic!(
                    "Expected to pop Character off Gateway. Popped something else:\n{:?}",
                    item
                ),
            }
        }
        Ok(())
    }
    fn current_moment(&self) -> Option<Clock::MomentRep> {
        self.last_seen_moment
    }
    fn is_empty(&self) -> bool {
        self.buffered_total == 0
    }
    fn next_is_character(&self) -> bool {
        match self.buffer[self.idx] {
            Self::InternalItem::Character(_) => true,
            _ => false,
        }
    }
    fn next_is_moment(&self) -> bool {
        match self.buffer[self.idx] {
            Self::InternalItem::Moment(_) => true,
            _ => false,
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum CharAscii {
    NullByte(),
    StartOfHeading(),
    StartOfText(),
    EndOfText(),
    EndOfTransmition(),
    Inquiry(),
    Ack(),
    Bel(),
    Backspace(),
    Tab(),
    LineFeed(),
    VerticalTab(),
    FormFeed(),
    CarriageReturn(),
    ShiftOut(),
    ShiftIn(),
    DataLinkEscape(),
    DeviceControl1(),
    DeviceControl2(),
    DeviceControl3(),
    DeviceControl4(),
    NegativeAck(),
    SyncIdle(),
    EndOfTransBlock(),
    Cancel(),
    EndOfMedium(),
    Substitute(),
    Escape(),
    FileSeparator(),
    GroupSeparator(),
    RecordSeparator(),
    UnitSeparator(),
    Space(),
    ExclamationPoint(),
    DoubleQuote(),
    PoundSign(),
    DollarSign(),
    PercentSign(),
    Ampersand(),
    SingleQuote(),
    OpenParenthesis(),
    CloseParenthesis(),
    StarSign(),
    PlusSign(),
    Comma(),
    MinusSign(),
    Period(),
    Slash(),
    Zero(),
    One(),
    Two(),
    Three(),
    Four(),
    Five(),
    Six(),
    Seven(),
    Eight(),
    Nine(),
    Colon(),
    Semicolon(),
    LessThanSign(),
    EqualsSign(),
    GreaterThanSign(),
    QuestionMark(),
    AtSign(),
    AUppercase(),
    BUppercase(),
    CUppercase(),
    DUppercase(),
    EUppercase(),
    FUppercase(),
    GUppercase(),
    HUppercase(),
    IUppercase(),
    JUppercase(),
    KUppercase(),
    LUppercase(),
    MUppercase(),
    NUppercase(),
    OUppercase(),
    PUppercase(),
    QUppercase(),
    RUppercase(),
    SUppercase(),
    TUppercase(),
    UUppercase(),
    VUppercase(),
    WUppercase(),
    XUppercase(),
    YUppercase(),
    ZUppercase(),
    SquareBracketLeft(),
    BackwardsSlash(),
    SquareBracketRight(),
    Caret(),
    Underscore(),
    BackTick(),
    ALowercase(),
    BLowercase(),
    CLowercase(),
    DLowercase(),
    ELowercase(),
    FLowercase(),
    GLowercase(),
    HLowercase(),
    ILowercase(),
    JLowercase(),
    KLowercase(),
    LLowercase(),
    MLowercase(),
    NLowercase(),
    OLowercase(),
    PLowercase(),
    QLowercase(),
    RLowercase(),
    SLowercase(),
    TLowercase(),
    ULowercase(),
    VLowercase(),
    WLowercase(),
    XLowercase(),
    YLowercase(),
    ZLowercase(),
    OpenCurlyBracket(),
    Pipe(),
    CloseCurlyBracket(),
    Tilde(),
    Delete(),
}
pub struct AlphabetAscii {}
impl AlphabetAscii {
    fn char_with_name(name: &str) -> Result<CharAscii, AlphabetError<&str>> {
        use CharAscii::*;
        match name {
            "NULL_BYTE" => Ok(NullByte()),
            "START_OF_HEADING" => Ok(StartOfHeading()),
            "START_OF_TEXT" => Ok(StartOfText()),
            "END_OF_TEXT" => Ok(EndOfText()),
            "END_OF_TRANSMITION" => Ok(EndOfTransmition()),
            "INQUIRY" => Ok(Inquiry()),
            "ACK" => Ok(Ack()),
            "BEL" => Ok(Bel()),
            "BACKSPACE" => Ok(Backspace()),
            "TAB" => Ok(Tab()),
            "LINE_FEED" => Ok(LineFeed()),
            "VERTICAL_TAB" => Ok(VerticalTab()),
            "FORM_FEED" => Ok(FormFeed()),
            "CARRIAGE_RETURN" => Ok(CarriageReturn()),
            "SHIFT_OUT" => Ok(ShiftOut()),
            "SHIFT_IN" => Ok(ShiftIn()),
            "DATA_LINK_ESCAPE" => Ok(DataLinkEscape()),
            "DEVICE_CONTROL_1" => Ok(DeviceControl1()),
            "DEVICE_CONTROL_2" => Ok(DeviceControl2()),
            "DEVICE_CONTROL_3" => Ok(DeviceControl3()),
            "DEVICE_CONTROL_4" => Ok(DeviceControl4()),
            "NEGATIVE_ACK" => Ok(NegativeAck()),
            "SYNC_IDLE" => Ok(SyncIdle()),
            "END_OF_TRANS_BLOCK" => Ok(EndOfTransBlock()),
            "CANCEL" => Ok(Cancel()),
            "END_OF_MEDIUM" => Ok(EndOfMedium()),
            "SUBSTITUTE" => Ok(Substitute()),
            "ESCAPE" => Ok(Escape()),
            "FILE_SEPARATOR" => Ok(FileSeparator()),
            "GROUP_SEPARATOR" => Ok(GroupSeparator()),
            "RECORD_SEPARATOR" => Ok(RecordSeparator()),
            "UNIT_SEPARATOR" => Ok(UnitSeparator()),
            "SPACE" => Ok(Space()),
            "EXCLAMATION_POINT" => Ok(ExclamationPoint()),
            "DOUBLE_QUOTE" => Ok(DoubleQuote()),
            "POUND_SIGN" => Ok(PoundSign()),
            "DOLLAR_SIGN" => Ok(DollarSign()),
            "PERCENT_SIGN" => Ok(PercentSign()),
            "AMPERSAND" => Ok(Ampersand()),
            "SINGLE_QUOTE" => Ok(SingleQuote()),
            "OPEN_PARENTHESIS" => Ok(OpenParenthesis()),
            "CLOSE_PARENTHESIS" => Ok(CloseParenthesis()),
            "STAR_SIGN" => Ok(StarSign()),
            "PLUS_SIGN" => Ok(PlusSign()),
            "COMMA" => Ok(Comma()),
            "MINUS_SIGN" => Ok(MinusSign()),
            "PERIOD" => Ok(Period()),
            "SLASH" => Ok(Slash()),
            "ZERO" => Ok(Zero()),
            "ONE" => Ok(One()),
            "TWO" => Ok(Two()),
            "THREE" => Ok(Three()),
            "FOUR" => Ok(Four()),
            "FIVE" => Ok(Five()),
            "SIX" => Ok(Six()),
            "SEVEN" => Ok(Seven()),
            "EIGHT" => Ok(Eight()),
            "NINE" => Ok(Nine()),
            "COLON" => Ok(Colon()),
            "SEMICOLON" => Ok(Semicolon()),
            "LESS_THAN_SIGN" => Ok(LessThanSign()),
            "EQUALS_SIGN" => Ok(EqualsSign()),
            "GREATER_THAN_SIGN" => Ok(GreaterThanSign()),
            "QUESTION_MARK" => Ok(QuestionMark()),
            "AT_SIGN" => Ok(AtSign()),
            "A_UPPERCASE" => Ok(AUppercase()),
            "B_UPPERCASE" => Ok(BUppercase()),
            "C_UPPERCASE" => Ok(CUppercase()),
            "D_UPPERCASE" => Ok(DUppercase()),
            "E_UPPERCASE" => Ok(EUppercase()),
            "F_UPPERCASE" => Ok(FUppercase()),
            "G_UPPERCASE" => Ok(GUppercase()),
            "H_UPPERCASE" => Ok(HUppercase()),
            "I_UPPERCASE" => Ok(IUppercase()),
            "J_UPPERCASE" => Ok(JUppercase()),
            "K_UPPERCASE" => Ok(KUppercase()),
            "L_UPPERCASE" => Ok(LUppercase()),
            "M_UPPERCASE" => Ok(MUppercase()),
            "N_UPPERCASE" => Ok(NUppercase()),
            "O_UPPERCASE" => Ok(OUppercase()),
            "P_UPPERCASE" => Ok(PUppercase()),
            "Q_UPPERCASE" => Ok(QUppercase()),
            "R_UPPERCASE" => Ok(RUppercase()),
            "S_UPPERCASE" => Ok(SUppercase()),
            "T_UPPERCASE" => Ok(TUppercase()),
            "U_UPPERCASE" => Ok(UUppercase()),
            "V_UPPERCASE" => Ok(VUppercase()),
            "W_UPPERCASE" => Ok(WUppercase()),
            "X_UPPERCASE" => Ok(XUppercase()),
            "Y_UPPERCASE" => Ok(YUppercase()),
            "Z_UPPERCASE" => Ok(ZUppercase()),
            "SQUARE_BRACKET_LEFT" => Ok(SquareBracketLeft()),
            "BACKWARDS_SLASH" => Ok(BackwardsSlash()),
            "SQUARE_BRACKET_RIGHT" => Ok(SquareBracketRight()),
            "CARET" => Ok(Caret()),
            "UNDERSCORE" => Ok(Underscore()),
            "BACK_TICK" => Ok(BackTick()),
            "A_LOWERCASE" => Ok(ALowercase()),
            "B_LOWERCASE" => Ok(BLowercase()),
            "C_LOWERCASE" => Ok(CLowercase()),
            "D_LOWERCASE" => Ok(DLowercase()),
            "E_LOWERCASE" => Ok(ELowercase()),
            "F_LOWERCASE" => Ok(FLowercase()),
            "G_LOWERCASE" => Ok(GLowercase()),
            "H_LOWERCASE" => Ok(HLowercase()),
            "I_LOWERCASE" => Ok(ILowercase()),
            "J_LOWERCASE" => Ok(JLowercase()),
            "K_LOWERCASE" => Ok(KLowercase()),
            "L_LOWERCASE" => Ok(LLowercase()),
            "M_LOWERCASE" => Ok(MLowercase()),
            "N_LOWERCASE" => Ok(NLowercase()),
            "O_LOWERCASE" => Ok(OLowercase()),
            "P_LOWERCASE" => Ok(PLowercase()),
            "Q_LOWERCASE" => Ok(QLowercase()),
            "R_LOWERCASE" => Ok(RLowercase()),
            "S_LOWERCASE" => Ok(SLowercase()),
            "T_LOWERCASE" => Ok(TLowercase()),
            "U_LOWERCASE" => Ok(ULowercase()),
            "V_LOWERCASE" => Ok(VLowercase()),
            "W_LOWERCASE" => Ok(WLowercase()),
            "X_LOWERCASE" => Ok(XLowercase()),
            "Y_LOWERCASE" => Ok(YLowercase()),
            "Z_LOWERCASE" => Ok(ZLowercase()),
            "OPEN_CURLY_BRACKET" => Ok(OpenCurlyBracket()),
            "PIPE" => Ok(Pipe()),
            "CLOSE_CURLY_BRACKET" => Ok(CloseCurlyBracket()),
            "TILDE" => Ok(Tilde()),
            "DELETE" => Ok(Delete()),
            _ => Err(AlphabetError::NameNotFound()),
        }
    }
    const fn to_char(rep: u8) -> Result<CharAscii, AlphabetError<u8>> {
        use CharAscii::*;
        match rep {
            0x0 => Ok(NullByte()),
            0x1 => Ok(StartOfHeading()),
            0x2 => Ok(StartOfText()),
            0x3 => Ok(EndOfText()),
            0x4 => Ok(EndOfTransmition()),
            0x5 => Ok(Inquiry()),
            0x6 => Ok(Ack()),
            0x7 => Ok(Bel()),
            0x8 => Ok(Backspace()),
            0x9 => Ok(Tab()),
            0xA => Ok(LineFeed()),
            0xB => Ok(VerticalTab()),
            0xC => Ok(FormFeed()),
            0xD => Ok(CarriageReturn()),
            0xE => Ok(ShiftOut()),
            0xF => Ok(ShiftIn()),
            0x10 => Ok(DataLinkEscape()),
            0x11 => Ok(DeviceControl1()),
            0x12 => Ok(DeviceControl2()),
            0x13 => Ok(DeviceControl3()),
            0x14 => Ok(DeviceControl4()),
            0x15 => Ok(NegativeAck()),
            0x16 => Ok(SyncIdle()),
            0x17 => Ok(EndOfTransBlock()),
            0x18 => Ok(Cancel()),
            0x19 => Ok(EndOfMedium()),
            0x1A => Ok(Substitute()),
            0x1B => Ok(Escape()),
            0x1C => Ok(FileSeparator()),
            0x1D => Ok(GroupSeparator()),
            0x1E => Ok(RecordSeparator()),
            0x1F => Ok(UnitSeparator()),
            0x20 => Ok(Space()),
            0x21 => Ok(ExclamationPoint()),
            0x22 => Ok(DoubleQuote()),
            0x23 => Ok(PoundSign()),
            0x24 => Ok(DollarSign()),
            0x25 => Ok(PercentSign()),
            0x26 => Ok(Ampersand()),
            0x27 => Ok(SingleQuote()),
            0x28 => Ok(OpenParenthesis()),
            0x29 => Ok(CloseParenthesis()),
            0x2A => Ok(StarSign()),
            0x2B => Ok(PlusSign()),
            0x2C => Ok(Comma()),
            0x2D => Ok(MinusSign()),
            0x2E => Ok(Period()),
            0x2F => Ok(Slash()),
            0x30 => Ok(Zero()),
            0x31 => Ok(One()),
            0x32 => Ok(Two()),
            0x33 => Ok(Three()),
            0x34 => Ok(Four()),
            0x35 => Ok(Five()),
            0x36 => Ok(Six()),
            0x37 => Ok(Seven()),
            0x38 => Ok(Eight()),
            0x39 => Ok(Nine()),
            0x3A => Ok(Colon()),
            0x3B => Ok(Semicolon()),
            0x3C => Ok(LessThanSign()),
            0x3D => Ok(EqualsSign()),
            0x3E => Ok(GreaterThanSign()),
            0x3F => Ok(QuestionMark()),
            0x40 => Ok(AtSign()),
            0x41 => Ok(AUppercase()),
            0x42 => Ok(BUppercase()),
            0x43 => Ok(CUppercase()),
            0x44 => Ok(DUppercase()),
            0x45 => Ok(EUppercase()),
            0x46 => Ok(FUppercase()),
            0x47 => Ok(GUppercase()),
            0x48 => Ok(HUppercase()),
            0x49 => Ok(IUppercase()),
            0x4A => Ok(JUppercase()),
            0x4B => Ok(KUppercase()),
            0x4C => Ok(LUppercase()),
            0x4D => Ok(MUppercase()),
            0x4E => Ok(NUppercase()),
            0x4F => Ok(OUppercase()),
            0x50 => Ok(PUppercase()),
            0x51 => Ok(QUppercase()),
            0x52 => Ok(RUppercase()),
            0x53 => Ok(SUppercase()),
            0x54 => Ok(TUppercase()),
            0x55 => Ok(UUppercase()),
            0x56 => Ok(VUppercase()),
            0x57 => Ok(WUppercase()),
            0x58 => Ok(XUppercase()),
            0x59 => Ok(YUppercase()),
            0x5A => Ok(ZUppercase()),
            0x5B => Ok(SquareBracketLeft()),
            0x5C => Ok(BackwardsSlash()),
            0x5D => Ok(SquareBracketRight()),
            0x5E => Ok(Caret()),
            0x5F => Ok(Underscore()),
            0x60 => Ok(BackTick()),
            0x61 => Ok(ALowercase()),
            0x62 => Ok(BLowercase()),
            0x63 => Ok(CLowercase()),
            0x64 => Ok(DLowercase()),
            0x65 => Ok(ELowercase()),
            0x66 => Ok(FLowercase()),
            0x67 => Ok(GLowercase()),
            0x68 => Ok(HLowercase()),
            0x69 => Ok(ILowercase()),
            0x6A => Ok(JLowercase()),
            0x6B => Ok(KLowercase()),
            0x6C => Ok(LLowercase()),
            0x6D => Ok(MLowercase()),
            0x6E => Ok(NLowercase()),
            0x6F => Ok(OLowercase()),
            0x70 => Ok(PLowercase()),
            0x71 => Ok(QLowercase()),
            0x72 => Ok(RLowercase()),
            0x73 => Ok(SLowercase()),
            0x74 => Ok(TLowercase()),
            0x75 => Ok(ULowercase()),
            0x76 => Ok(VLowercase()),
            0x77 => Ok(WLowercase()),
            0x78 => Ok(XLowercase()),
            0x79 => Ok(YLowercase()),
            0x7A => Ok(ZLowercase()),
            0x7B => Ok(OpenCurlyBracket()),
            0x7C => Ok(Pipe()),
            0x7D => Ok(CloseCurlyBracket()),
            0x7E => Ok(Tilde()),
            0x7F => Ok(Delete()),
            _ => Err(AlphabetError::UnknownCharacter(rep)),
        }
    }
    const fn to_val(chr: CharAscii) -> u8 {
        use CharAscii::*;
        match chr {
            NullByte() => 0x0 as u8,
            StartOfHeading() => 0x1 as u8,
            StartOfText() => 0x2 as u8,
            EndOfText() => 0x3 as u8,
            EndOfTransmition() => 0x4 as u8,
            Inquiry() => 0x5 as u8,
            Ack() => 0x6 as u8,
            Bel() => 0x7 as u8,
            Backspace() => 0x8 as u8,
            Tab() => 0x9 as u8,
            LineFeed() => 0xA as u8,
            VerticalTab() => 0xB as u8,
            FormFeed() => 0xC as u8,
            CarriageReturn() => 0xD as u8,
            ShiftOut() => 0xE as u8,
            ShiftIn() => 0xF as u8,
            DataLinkEscape() => 0x10 as u8,
            DeviceControl1() => 0x11 as u8,
            DeviceControl2() => 0x12 as u8,
            DeviceControl3() => 0x13 as u8,
            DeviceControl4() => 0x14 as u8,
            NegativeAck() => 0x15 as u8,
            SyncIdle() => 0x16 as u8,
            EndOfTransBlock() => 0x17 as u8,
            Cancel() => 0x18 as u8,
            EndOfMedium() => 0x19 as u8,
            Substitute() => 0x1A as u8,
            Escape() => 0x1B as u8,
            FileSeparator() => 0x1C as u8,
            GroupSeparator() => 0x1D as u8,
            RecordSeparator() => 0x1E as u8,
            UnitSeparator() => 0x1F as u8,
            Space() => 0x20 as u8,
            ExclamationPoint() => 0x21 as u8,
            DoubleQuote() => 0x22 as u8,
            PoundSign() => 0x23 as u8,
            DollarSign() => 0x24 as u8,
            PercentSign() => 0x25 as u8,
            Ampersand() => 0x26 as u8,
            SingleQuote() => 0x27 as u8,
            OpenParenthesis() => 0x28 as u8,
            CloseParenthesis() => 0x29 as u8,
            StarSign() => 0x2A as u8,
            PlusSign() => 0x2B as u8,
            Comma() => 0x2C as u8,
            MinusSign() => 0x2D as u8,
            Period() => 0x2E as u8,
            Slash() => 0x2F as u8,
            Zero() => 0x30 as u8,
            One() => 0x31 as u8,
            Two() => 0x32 as u8,
            Three() => 0x33 as u8,
            Four() => 0x34 as u8,
            Five() => 0x35 as u8,
            Six() => 0x36 as u8,
            Seven() => 0x37 as u8,
            Eight() => 0x38 as u8,
            Nine() => 0x39 as u8,
            Colon() => 0x3A as u8,
            Semicolon() => 0x3B as u8,
            LessThanSign() => 0x3C as u8,
            EqualsSign() => 0x3D as u8,
            GreaterThanSign() => 0x3E as u8,
            QuestionMark() => 0x3F as u8,
            AtSign() => 0x40 as u8,
            AUppercase() => 0x41 as u8,
            BUppercase() => 0x42 as u8,
            CUppercase() => 0x43 as u8,
            DUppercase() => 0x44 as u8,
            EUppercase() => 0x45 as u8,
            FUppercase() => 0x46 as u8,
            GUppercase() => 0x47 as u8,
            HUppercase() => 0x48 as u8,
            IUppercase() => 0x49 as u8,
            JUppercase() => 0x4A as u8,
            KUppercase() => 0x4B as u8,
            LUppercase() => 0x4C as u8,
            MUppercase() => 0x4D as u8,
            NUppercase() => 0x4E as u8,
            OUppercase() => 0x4F as u8,
            PUppercase() => 0x50 as u8,
            QUppercase() => 0x51 as u8,
            RUppercase() => 0x52 as u8,
            SUppercase() => 0x53 as u8,
            TUppercase() => 0x54 as u8,
            UUppercase() => 0x55 as u8,
            VUppercase() => 0x56 as u8,
            WUppercase() => 0x57 as u8,
            XUppercase() => 0x58 as u8,
            YUppercase() => 0x59 as u8,
            ZUppercase() => 0x5A as u8,
            SquareBracketLeft() => 0x5B as u8,
            BackwardsSlash() => 0x5C as u8,
            SquareBracketRight() => 0x5D as u8,
            Caret() => 0x5E as u8,
            Underscore() => 0x5F as u8,
            BackTick() => 0x60 as u8,
            ALowercase() => 0x61 as u8,
            BLowercase() => 0x62 as u8,
            CLowercase() => 0x63 as u8,
            DLowercase() => 0x64 as u8,
            ELowercase() => 0x65 as u8,
            FLowercase() => 0x66 as u8,
            GLowercase() => 0x67 as u8,
            HLowercase() => 0x68 as u8,
            ILowercase() => 0x69 as u8,
            JLowercase() => 0x6A as u8,
            KLowercase() => 0x6B as u8,
            LLowercase() => 0x6C as u8,
            MLowercase() => 0x6D as u8,
            NLowercase() => 0x6E as u8,
            OLowercase() => 0x6F as u8,
            PLowercase() => 0x70 as u8,
            QLowercase() => 0x71 as u8,
            RLowercase() => 0x72 as u8,
            SLowercase() => 0x73 as u8,
            TLowercase() => 0x74 as u8,
            ULowercase() => 0x75 as u8,
            VLowercase() => 0x76 as u8,
            WLowercase() => 0x77 as u8,
            XLowercase() => 0x78 as u8,
            YLowercase() => 0x79 as u8,
            ZLowercase() => 0x7A as u8,
            OpenCurlyBracket() => 0x7B as u8,
            Pipe() => 0x7C as u8,
            CloseCurlyBracket() => 0x7D as u8,
            Tilde() => 0x7E as u8,
            Delete() => 0x7F as u8,
        }
    }
}
impl AlphabetLike for AlphabetAscii {
    type CharRep = u8;
    type CharEnum = CharAscii;
    fn char_with_name(name: &str) -> Result<CharAscii, AlphabetError<&str>> {
        <AlphabetAscii>::char_with_name(name)
    }
    fn to_char(rep: u8) -> Result<CharAscii, AlphabetError<u8>> {
        <AlphabetAscii>::to_char(rep)
    }
    fn to_val(chr: CharAscii) -> u8 {
        <AlphabetAscii>::to_val(chr)
    }
}

pub struct ClockCounterClock {}
impl ClockCounterClock {
    const fn to_moment(rep: u32) -> ClockMoment<u32> {
        ClockMoment::Quantity(rep)
    }
    const fn represents() -> &'static str {
        "QUANTITY"
    }
}
impl ClockLike for ClockCounterClock {
    type MomentRep = u32;
    fn represents(&self) -> &str {
        <ClockCounterClock>::represents()
    }
    fn to_moment(rep: u32) -> ClockMoment<u32> {
        <ClockCounterClock>::to_moment(rep)
    }
}
impl AddableClockLike<u32> for ClockCounterClock {}

pub struct ProgramHelloWorld {
    pub exit_a: Stream<AlphabetAscii, ClockCounterClock, 0x50>,
}
impl ProgramHelloWorld {
    pub const fn new() -> Self {
        Self {
            exit_a: <Stream<AlphabetAscii, ClockCounterClock, 0x50>>::new(),
        }
    }
    pub fn label_root(&mut self) {
        self.exit_a.set_initial_moment(0);
        self.exit_a
            .push_moment(1)
            .expect("Could not push_moment to Exit (A)");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::HUppercase())
            .expect("Could not push_char (\"H_UPPERCASE\")");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::ELowercase())
            .expect("Could not push_char (\"E_LOWERCASE\")");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::LLowercase())
            .expect("Could not push_char (\"L_LOWERCASE\")");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::LLowercase())
            .expect("Could not push_char (\"L_LOWERCASE\")");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::OLowercase())
            .expect("Could not push_char (\"O_LOWERCASE\")");
        self.exit_a
            .push(
                AlphabetAscii::to_char(0x2C)
                    .expect("No character found in Alphabet (ASCII): \"0x2C\""),
            )
            .expect("Could not push_val to Exit (A)");
        self.exit_a
            .push(
                AlphabetAscii::to_char(0x20)
                    .expect("No character found in Alphabet (ASCII): \"0x20\""),
            )
            .expect("Could not push_val to Exit (A)");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::WUppercase())
            .expect("Could not push_char (\"W_UPPERCASE\")");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::OLowercase())
            .expect("Could not push_char (\"O_LOWERCASE\")");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::RLowercase())
            .expect("Could not push_char (\"R_LOWERCASE\")");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::LLowercase())
            .expect("Could not push_char (\"L_LOWERCASE\")");
        self.exit_a
            .push(<AlphabetAscii as AlphabetLike>::CharEnum::DLowercase())
            .expect("Could not push_char (\"D_LOWERCASE\")");
        self.exit_a
            .push(
                AlphabetAscii::to_char(0x21)
                    .expect("No character found in Alphabet (ASCII): \"0x21\""),
            )
            .expect("Could not push_val to Exit (A)");
        self.exit_a
            .push_moment(1)
            .expect("Could not push_moment to Exit (A)");
    }
}

pub struct ProgramSync2 {
    pub gateway_a: Stream<AlphabetAscii, ClockCounterClock, 0x50>,
    pub gateway_b: Stream<AlphabetAscii, ClockCounterClock, 0x50>,
    pub exit_c: Stream<AlphabetAscii, ClockCounterClock, 0x50>,
    pub exit_d: Stream<AlphabetAscii, ClockCounterClock, 0x50>,
}
impl ProgramSync2 {
    pub const fn new() -> Self {
        Self {
            gateway_a: <Stream<AlphabetAscii, ClockCounterClock, 0x50>>::new(),
            gateway_b: <Stream<AlphabetAscii, ClockCounterClock, 0x50>>::new(),
            exit_c: <Stream<AlphabetAscii, ClockCounterClock, 0x50>>::new(),
            exit_d: <Stream<AlphabetAscii, ClockCounterClock, 0x50>>::new(),
        }
    }
    pub fn label_root(&mut self) {}
    pub fn label_main(&mut self) {
        if ClockCounterClock::represents() != ClockCounterClock::represents() {
            panic ! ("(Clock of) Gateway A and (Clock of) Gateway B being compared while not representing the same thing");
        }
        match (
            self.gateway_a.current_moment(),
            self.gateway_b.current_moment(),
        ) {
            (None, Some(_)) => {
                return self.label_a_earlier();
            }
            (Some(a), Some(b)) if a < b => {
                return self.label_a_earlier();
            }
            _ => (),
        }
        if ClockCounterClock::represents() != ClockCounterClock::represents() {
            panic ! ("(Clock of) Gateway A and (Clock of) Gateway B being compared while not representing the same thing");
        }
        match (
            self.gateway_a.current_moment(),
            self.gateway_b.current_moment(),
        ) {
            (Some(_), None) => {
                return self.label_a_later();
            }
            (Some(a), Some(b)) if a > b => {
                return self.label_a_later();
            }
            _ => (),
        }
        loop {
            match self.gateway_a.pop() {
                StreamItem::Character(chr) => {
                    self.exit_c
                        .push(chr)
                        .expect("Failed to forward character from Gateway A to Exit C");
                }
                StreamItem::Moment(moment) => {
                    self.exit_c
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway A to Exit C");
                    break;
                }
                StreamItem::Empty => continue,
            }
        }
        if self.gateway_a.next_is_moment() {
            match self.gateway_a.pop() {
                StreamItem::Moment(moment) => {
                    self.exit_c
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway A to Exit C");
                }
                _ => {
                    panic ! ("Unreachable Code - unexpectedly popped a non-moment when calling forward_moment()");
                }
            }
        } else {
            panic ! ("Tried to forward_moment from {} to {} when the next item in the gateway, is not a Moment" , "A" , "C")
        }
        loop {
            match self.gateway_b.pop() {
                StreamItem::Character(chr) => {
                    self.exit_d
                        .push(chr)
                        .expect("Failed to forward character from Gateway B to Exit D");
                }
                StreamItem::Moment(moment) => {
                    self.exit_d
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway B to Exit D");
                    break;
                }
                StreamItem::Empty => continue,
            }
        }
        if self.gateway_b.next_is_moment() {
            match self.gateway_b.pop() {
                StreamItem::Moment(moment) => {
                    self.exit_d
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway B to Exit D");
                }
                _ => {
                    panic ! ("Unreachable Code - unexpectedly popped a non-moment when calling forward_moment()");
                }
            }
        } else {
            panic ! ("Tried to forward_moment from {} to {} when the next item in the gateway, is not a Moment" , "B" , "D")
        }
    }
    pub fn label_a_earlier(&mut self) {
        if self.gateway_a.next_is_moment() {
            match self.gateway_a.pop() {
                StreamItem::Moment(moment) => {
                    self.exit_d
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway A to Exit D");
                }
                _ => {
                    panic ! ("Unreachable Code - unexpectedly popped a non-moment when calling forward_moment()");
                }
            }
        } else {
            panic ! ("Tried to forward_moment from {} to {} when the next item in the gateway, is not a Moment" , "A" , "D")
        }
        loop {
            match self.gateway_a.pop() {
                StreamItem::Character(chr) => {
                    self.exit_c
                        .push(chr)
                        .expect("Failed to forward character from Gateway A to Exit C");
                }
                StreamItem::Moment(moment) => {
                    self.exit_c
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway A to Exit C");
                    break;
                }
                StreamItem::Empty => continue,
            }
        }
        if self.gateway_a.next_is_moment() {
            match self.gateway_a.pop() {
                StreamItem::Moment(moment) => {
                    self.exit_c
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway A to Exit C");
                }
                _ => {
                    panic ! ("Unreachable Code - unexpectedly popped a non-moment when calling forward_moment()");
                }
            }
        } else {
            panic ! ("Tried to forward_moment from {} to {} when the next item in the gateway, is not a Moment" , "A" , "C")
        }
    }
    pub fn label_a_later(&mut self) {
        if self.gateway_b.next_is_moment() {
            match self.gateway_b.pop() {
                StreamItem::Moment(moment) => {
                    self.exit_c
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway B to Exit C");
                }
                _ => {
                    panic ! ("Unreachable Code - unexpectedly popped a non-moment when calling forward_moment()");
                }
            }
        } else {
            panic ! ("Tried to forward_moment from {} to {} when the next item in the gateway, is not a Moment" , "B" , "C")
        }
        loop {
            match self.gateway_b.pop() {
                StreamItem::Character(chr) => {
                    self.exit_d
                        .push(chr)
                        .expect("Failed to forward character from Gateway B to Exit D");
                }
                StreamItem::Moment(moment) => {
                    self.exit_d
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway B to Exit D");
                    break;
                }
                StreamItem::Empty => continue,
            }
        }
        if self.gateway_b.next_is_moment() {
            match self.gateway_b.pop() {
                StreamItem::Moment(moment) => {
                    self.exit_d
                        .push_moment(moment)
                        .expect("Failed to forward moment from Gateway B to Exit D");
                }
                _ => {
                    panic ! ("Unreachable Code - unexpectedly popped a non-moment when calling forward_moment()");
                }
            }
        } else {
            panic ! ("Tried to forward_moment from {} to {} when the next item in the gateway, is not a Moment" , "B" , "D")
        }
    }
}

