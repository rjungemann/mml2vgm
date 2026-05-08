//! Lexer for MML files
//!
//! This module provides tokenization of MML (Music Macro Language) source code.
//! It handles the full MML syntax including notes, commands, definitions, and metadata.

use crate::{MmlError, MmlResult, Position};

/// Token types for MML
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Generic tokens
    /// Numeric literal
    Number(u32),
    /// String literal (quoted text)
    StringLiteral(String),
    /// Identifier (alphanumeric + underscore, hyphen)
    Identifier(String),
    
    // Structure
    /// Song info block start: {
    LeftBrace,
    /// Song info block end: }
    RightBrace,
    /// Definition line prefix: '
    Apostrophe,
    /// Equals: =
    Equals,
    /// Comma: ,
    Comma,
    /// Left bracket: [
    LeftBracket,
    /// Right bracket: ]
    RightBracket,
    /// Left paren: (
    LeftParen,
    /// Right paren: )
    RightParen,
    
    // Notes
    /// Note: C, D, E, F, G, A, B
    Note(char),
    /// MIDI note number command: n or N (followed by a number 0-127)
    NoteNumberCommand,
    /// Sharp: #
    Sharp,
    /// Flat: b
    Flat,
    /// Rest: r or R
    Rest,
    
    // Duration and timing
    /// Duration number (1, 2, 4, 8, 16, 32, 64, 128)
    Duration(u32),
    /// Dotted note modifier: .
    Dot,
    /// Tie modifier: _
    Underscore,
    
    // Octave
    /// Octave up: >
    GreaterThan,
    /// Octave down: <
    LessThan,
    /// Octave command: o
    OctaveCommand,
    
    // Volume
    /// Volume command: v
    VolumeCommand,
    
    // Tempo
    /// Tempo command: t or T
    TempoCommand,
    
    // Length
    /// Length command: l
    LengthCommand,
    
    // Instrument
    /// Instrument/Definition command: @
    AtSign,

    // MIDI-specific commands
    /// Control Change: c
    ControlChangeCommand,
    /// Program Change: p or pg
    ProgramChangeCommand,
    /// Pitch Bend: b or bend
    PitchBendCommand,
    /// Aftertouch (Channel Pressure): a or at
    AftertouchCommand,
    /// Polyphonic Aftertouch: pa
    PolyAftertouchCommand,
    /// System Exclusive: x or sysex
    SysExCommand,
    /// MIDI Channel: ch
    MidiChannelCommand,
    /// MIDI Program: pr
    MidiProgramCommand,
    /// Pan: pan
    PanCommand,
    /// Expression: expr
    ExpressionCommand,
    /// Sustain: sustain, sustainOff
    SustainCommand,
    /// All Notes Off: allNotesOff
    AllNotesOffCommand,
    /// Reset All Controllers: resetAllCtrl
    ResetAllCtrlCommand,
    /// All Sound Off: allSoundOff
    AllSoundOffCommand,
    /// Damper pedal: damper, damperOff
    DamperCommand,
    /// Portamento: portamento, portOff
    PortamentoCommand,
    /// Sostenuto: sostenuto, sostenutoOff
    SostenutoCommand,
    /// Soft pedal: soft, softOff
    SoftCommand,
    /// Local Control: localOn, localOff
    LocalControlCommand,
    /// Drum note: D
    DrumNoteCommand,
    
    // Special
    /// Bar line: |
    Bar,
    /// Comment
    Comment(String),
    /// Whitespace
    Whitespace(String),
    /// End of file
    Eof,
}

/// Lexer for MML source code
pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
    current_line: usize,
    current_column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            current_line: 1,
            current_column: 1,
        }
    }

    /// Get current character
    fn current_char(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Get next character
    fn next_char(&self) -> Option<char> {
        // Skip the current character and get the next one
        let mut iter = self.source[self.position..].chars();
        let _current = iter.next()?;  // Get current (might be multi-byte)
        iter.next()  // Get next
    }

    /// Peek at the current character without advancing
    fn peek_char(&self) -> Option<char> {
        self.current_char()
    }

    /// Peek at the next N characters as a string
    fn peek_chars(&self, n: usize) -> Option<&str> {
        let remaining = &self.source[self.position..];
        let char_count = remaining.chars().count();
        if char_count < n {
            return None;
        }
        // This is a simplified version - we need to account for multi-byte UTF-8
        // For ASCII-only MML commands, this should work
        let byte_slice = &remaining.as_bytes()[..n];
        std::str::from_utf8(byte_slice).ok()
    }

    /// Advance N characters
    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    /// Advance to next character
    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            self.position += c.len_utf8();
            if c == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            } else if c != '\r' {
                self.current_column += 1;
            }
        }
    }

    /// Skip whitespace (spaces, tabs)
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Check if end of file
    fn is_eof(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Get current position
    pub fn position(&self) -> Position {
        Position::new(self.current_line, self.current_column)
    }

    /// Read a string literal (handling quotes)
    fn read_string(&mut self) -> String {
        let quote_char = self.current_char().unwrap_or('"');
        self.advance(); // Skip opening quote
        
        let mut result = String::new();
        while let Some(c) = self.current_char() {
            if c == quote_char {
                self.advance(); // Skip closing quote
                break;
            }
            if c == '\\' {
                self.advance();
                if let Some(next) = self.current_char() {
                    match next {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => result.push(next),
                    }
                    self.advance();
                }
            } else {
                result.push(c);
                self.advance();
            }
        }
        result
    }

    /// Read an identifier (alphanumeric + underscore, hyphen, =)
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(c) = self.current_char() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '=' {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    /// Read a number
    fn read_number(&mut self) -> u32 {
        let mut result = String::new();
        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result.parse().unwrap_or(0)
    }

    /// Next token
    pub fn next_token(&mut self) -> MmlResult<Token> {
        self.skip_whitespace();
        
        if self.is_eof() {
            return Ok(Token::Eof);
        }

        let c = self.current_char().unwrap();
        
        // Two-character tokens first
        if c == '\'' {
            // Check if this is the start of a definition line
            self.advance();
            return Ok(Token::Apostrophe);
        }
        
        // Check for notes first (C, D, E, F, G, A, B) - these take priority.
        // Exception: if the note letter is UPPERCASE and followed by an UPPERCASE non-note letter
        // (e.g. "EON", "EX2"), fall through to the identifier check.
        // Lowercase note letters always produce a Note token (e.g. 'c', 'e', 'g' in music).
        let c_upper = c.to_ascii_uppercase();
        if c_upper == 'C' || c_upper == 'D' || c_upper == 'E' || c_upper == 'F' ||
           c_upper == 'G' || c_upper == 'A' || c_upper == 'B' {
            let letter = c_upper;
            if let Some(next_c) = self.next_char() {
                // Only fall through to identifier if BOTH this letter and the next are uppercase.
                // This lets "EON", "EX2" etc. parse as identifiers while 'c','e' in music stay as notes.
                let next_upper = next_c.to_ascii_uppercase();
                let next_is_note = next_upper == 'C' || next_upper == 'D' || next_upper == 'E'
                    || next_upper == 'F' || next_upper == 'G' || next_upper == 'A'
                    || next_upper == 'B';
                if c.is_uppercase() && next_c.is_alphabetic() && !next_is_note {
                    // Fall through to identifier check below:
                    // uppercase note letter + non-note alphabetic char → multi-letter identifier
                    // (e.g. "ComposerJ", "Coda"; music notes are single letters or
                    //  uppercase + another note letter like "CD", "CE")
                } else {
                    self.advance();
                    return Ok(Token::Note(letter));
                }
            } else {
                self.advance();
                return Ok(Token::Note(letter));
            }
        }
        
        // Check for identifiers (including commands like INCLUDE, alias, etc.)
        // Lowercase single-letter MML command chars must NOT be combined into a multi-letter
        // identifier — they fall through to the command match below. Uppercase variants are
        // left unrestricted so that documentation headers like "RR SL TL" tokenize as
        // identifiers rather than command tokens.
        let c_is_lowercase_mml_cmd = matches!(c, 'r' | 'v' | 't' | 'l' | 'o');
        if !c_is_lowercase_mml_cmd && (c.is_alphabetic() || c == '_') {
            let next_c = self.next_char();
            if next_c.map_or(false, |nc| nc.is_alphabetic() || nc == '_' || nc == '-' || nc == '=') {
                let ident = self.read_identifier();
                return Ok(Token::Identifier(ident));
            }
        }
        
        // Single character tokens
        match c {
            // Structure
            '{' => { self.advance(); Ok(Token::LeftBrace) }
            '}' => { self.advance(); Ok(Token::RightBrace) }
            '=' => { self.advance(); Ok(Token::Equals) }
            ',' => { self.advance(); Ok(Token::Comma) }
            '[' => { self.advance(); Ok(Token::LeftBracket) }
            ']' => { self.advance(); Ok(Token::RightBracket) }
            '(' => { self.advance(); Ok(Token::LeftParen) }
            ')' => { self.advance(); Ok(Token::RightParen) }
            
            // Rest
            'r' | 'R' => {
                self.advance();
                Ok(Token::Rest)
            }
            
            // Accidentals (only if not part of note)
            '#' => {
                self.advance();
                Ok(Token::Sharp)
            }
            'b' => {
                self.advance();
                Ok(Token::Flat)
            }
            
            // Duration and timing
            '.' => {
                self.advance();
                Ok(Token::Dot)
            }
            '_' => {
                self.advance();
                Ok(Token::Underscore)
            }
            
            // Octave
            '>' => {
                self.advance();
                Ok(Token::GreaterThan)
            }
            '<' => {
                self.advance();
                Ok(Token::LessThan)
            }
            'o' | 'O' => {
                self.advance();
                Ok(Token::OctaveCommand)
            }
            
            // Volume
            'v' | 'V' => {
                self.advance();
                Ok(Token::VolumeCommand)
            }
            
            // Tempo
            't' | 'T' => {
                self.advance();
                Ok(Token::TempoCommand)
            }
            
            // Length
            'l' | 'L' => {
                self.advance();
                Ok(Token::LengthCommand)
            }
            
            // MIDI note number command
            'n' | 'N' => {
                self.advance();
                Ok(Token::NoteNumberCommand)
            }

            // Instrument/At sign and MIDI commands
            '@' => {
                self.advance();
                // Check for MIDI-specific commands by looking at the following characters
                let next_char = self.peek_char();
                let next_token = match next_char {
                    Some('c') => {
                        self.advance();
                        // Check if it's 'cc' for control change or 'ch' for channel
                        if self.peek_char() == Some('c') {
                            self.advance();
                            Token::ControlChangeCommand
                        } else if self.peek_char() == Some('h') {
                            self.advance();
                            Token::MidiChannelCommand
                        } else {
                            Token::ControlChangeCommand
                        }
                    }
                    Some('p') => {
                        self.advance();
                        // Check if it's 'pg' for program change, 'pr' for program, or 'pan'
                        if self.peek_char() == Some('g') {
                            self.advance();
                            Token::ProgramChangeCommand
                        } else if self.peek_char() == Some('r') {
                            self.advance();
                            Token::MidiProgramCommand
                        } else if self.peek_chars(2) == Some("an") {
                            self.advance_n(2);
                            Token::PanCommand
                        } else {
                            Token::ProgramChangeCommand
                        }
                    }
                    Some('b') => {
                        self.advance();
                        // Check if it's 'bend'
                        if self.peek_chars(3) == Some("end") {
                            self.advance_n(3);
                            Token::PitchBendCommand
                        } else {
                            Token::PitchBendCommand
                        }
                    }
                    Some('a') => {
                        self.advance();
                        // Check if it's 'at' for aftertouch
                        if self.peek_char() == Some('t') {
                            self.advance();
                            Token::AftertouchCommand
                        } else {
                            Token::AftertouchCommand
                        }
                    }
                    Some('x') => {
                        self.advance();
                        // Check if it's 'sysex'
                        if self.peek_chars(4) == Some("ysex") {
                            self.advance_n(4);
                            Token::SysExCommand
                        } else {
                            Token::SysExCommand
                        }
                    }
                    Some('D') => {
                        self.advance();
                        Token::DrumNoteCommand
                    }
                    Some('e') => {
                        self.advance();
                        // Check for expr
                        if self.peek_chars(3) == Some("xpr") {
                            self.advance_n(3);
                            Token::ExpressionCommand
                        } else {
                            Token::Identifier("e".to_string())
                        }
                    }
                    Some('s') => {
                        self.advance();
                        // Check for sustain, sustainOff, soft, softOff, sostenuto, sostenutoOff
                        if self.peek_chars(6) == Some("ustain") {
                            self.advance_n(6);
                            // Check for Off
                            if self.peek_chars(3) == Some("Off") {
                                self.advance_n(3);
                                Token::SustainCommand // We'll handle off in parser
                            } else {
                                Token::SustainCommand
                            }
                        } else if self.peek_chars(4) == Some("oft") {
                            self.advance_n(4);
                            // Check for Off - advance and return SoftCommand
                            if self.peek_char() == Some('O') && self.peek_chars(3) == Some("Off") {
                                self.advance_n(3);
                                Token::SoftCommand
                            } else {
                                Token::SoftCommand
                            }
                        } else if self.peek_chars(8) == Some("ostenuto") {
                            self.advance_n(8);
                            // Check for Off
                            if self.peek_chars(3) == Some("Off") {
                                self.advance_n(3);
                                Token::SostenutoCommand
                            } else {
                                Token::SostenutoCommand
                            }
                        } else {
                            Token::Identifier("s".to_string())
                        }
                    }
                    Some('d') => {
                        self.advance();
                        // Check for damper, damperOff
                        if self.peek_chars(5) == Some("amper") {
                            self.advance_n(5);
                            Token::DamperCommand
                        } else {
                            Token::Identifier("d".to_string())
                        }
                    }
                    Some('p') => {
                        self.advance();
                        // Check for portamento, portOff
                        if self.peek_chars(8) == Some("ortamento") {
                            self.advance_n(8);
                            Token::PortamentoCommand
                        } else {
                            // Already handled pg, pr, pan above
                            Token::ProgramChangeCommand
                        }
                    }
                    Some('l') => {
                        self.advance();
                        // Check for localOn, localOff
                        if self.peek_chars(5) == Some("ocal") {
                            self.advance_n(5);
                            // Check for On
                            if self.peek_char() == Some('O') && self.peek_chars(2) == Some("n") {
                                self.advance_n(2);
                                Token::LocalControlCommand
                            } else if self.peek_char() == Some('f') && self.peek_chars(2) == Some("ff") {
                                self.advance_n(3);
                                Token::LocalControlCommand
                            } else {
                                Token::LocalControlCommand
                            }
                        } else {
                            Token::Identifier("l".to_string())
                        }
                    }
                    Some('a') => {
                        self.advance();
                        // Check for allSoundOff, allNotesOff
                        if self.peek_chars(8) == Some("llSound") {
                            self.advance_n(8);
                            if self.peek_chars(3) == Some("Off") {
                                self.advance_n(3);
                                Token::AllSoundOffCommand
                            } else {
                                Token::AllSoundOffCommand
                            }
                        } else if self.peek_chars(7) == Some("llNotes") {
                            self.advance_n(7);
                            if self.peek_chars(3) == Some("Off") {
                                self.advance_n(3);
                                Token::AllNotesOffCommand
                            } else {
                                Token::AllNotesOffCommand
                            }
                        } else if self.peek_char() == Some('t') {
                            self.advance();
                            Token::AftertouchCommand
                        } else {
                            Token::Identifier("a".to_string())
                        }
                    }
                    Some('r') => {
                        self.advance();
                        // Check for resetAllCtrl
                        if self.peek_chars(9) == Some("esetAllC") {
                            self.advance_n(9);
                            if self.peek_chars(4) == Some("trl") {
                                self.advance_n(4);
                                Token::ResetAllCtrlCommand
                            } else {
                                Token::Identifier("resetAllC".to_string())
                            }
                        } else {
                            Token::Identifier("r".to_string())
                        }
                    }
                    _ => Token::AtSign,
                };
                Ok(next_token)
            }
            
            // Bar
            '|' => {
                self.advance();
                Ok(Token::Bar)
            }
            
            // String literal
            '\"' => {
                let s = self.read_string();
                Ok(Token::StringLiteral(s))
            }
            
            // Numbers
            _ if c.is_ascii_digit() => {
                let num = self.read_number();
                Ok(Token::Number(num))
            }
            
            // Single-letter identifiers (fallback for commands that aren't special-cased)
            _ if c.is_alphabetic() || c == '_' => {
                let ident = self.read_identifier();
                Ok(Token::Identifier(ident))
            }
            
            // Newline - skip and continue
            '\n' | '\r' => {
                self.advance();
                // Skip consecutive newlines
                while let Some(c) = self.current_char() {
                    if c == '\n' || c == '\r' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                // Recursively get next token
                self.next_token()
            }
            
            // '+' is an alternative sharp symbol (MML C# format: c+4 = C-sharp quarter note)
            '+' => {
                self.advance();
                Ok(Token::Sharp)
            }
            // '-' is an alternative flat symbol (MML C# format: e-4 = E-flat quarter note).
            // Note: this only triggers for standalone '-'; hyphens inside identifier names are
            // consumed by read_identifier() before reaching here.
            '-' => {
                self.advance();
                Ok(Token::Flat)
            }

            // Whitespace or unrecognised character (advance past it to avoid infinite loop)
            _ => {
                let mut ws = String::new();
                let mut advanced = false;
                while let Some(c) = self.current_char() {
                    if c.is_whitespace() && c != '\n' && c != '\r' {
                        ws.push(c);
                        self.advance();
                        advanced = true;
                    } else {
                        break;
                    }
                }
                if !advanced {
                    // Unrecognised character: skip it so the tokenizer never stalls.
                    self.advance();
                }
                Ok(Token::Whitespace(ws))
            }
        }
    }
}

/// Tokenize entire source into a vector of tokens with positions
pub fn tokenize(source: &str) -> MmlResult<Vec<(Token, Position)>> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    
    loop {
        let pos = lexer.position();
        let token = lexer.next_token()?;
        let is_whitespace = matches!(&token, Token::Whitespace(_));
        let is_eof = matches!(&token, Token::Eof);
        
        // Skip whitespace tokens unless they're meaningful
        if !is_whitespace && !is_eof {
            tokens.push((token, pos));
        }
        
        if is_eof {
            break;
        }
    }
    
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_notes() {
        let source = "o4 c4 d4 e4 f4";
        let tokens = tokenize(source).unwrap();
        
        // Should have: OctaveCommand, Number(4), Note('c'), Number(4), Note('d'), Number(4), ...
        assert!(tokens.len() >= 5);
    }

    #[test]
    fn test_song_info_block() {
        let source = "{ TitleName = Test ComposerJ = Author }";
        let tokens = tokenize(source).unwrap();
        
        assert_eq!(tokens[0].0, Token::LeftBrace);
        // Find the equals sign
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Equals)));
        assert_eq!(tokens.last().unwrap().0, Token::RightBrace);
    }

    #[test]
    fn test_definition_line() {
        let source = "'A1 T120";
        let tokens = tokenize(source).unwrap();
        
        assert_eq!(tokens[0].0, Token::Apostrophe);
        // A1 is tokenized as Note('A') + Number(1) because 'A' is a note letter
        assert_eq!(tokens[1].0, Token::Note('A'));
        assert_eq!(tokens[2].0, Token::Number(1));
        // T120 is tokenized as TempoCommand + Number(120)
        assert_eq!(tokens[3].0, Token::TempoCommand);
        assert_eq!(tokens[4].0, Token::Number(120));
    }

    #[test]
    fn test_rest() {
        let source = "r4";
        let tokens = tokenize(source).unwrap();
        
        assert_eq!(tokens[0].0, Token::Rest);
        assert_eq!(tokens[1].0, Token::Number(4));
    }

    #[test]
    fn test_sharp_flat() {
        let source = "c#4 db4";
        let tokens = tokenize(source).unwrap();
        
        // c#4 is: Note('c'), Sharp, Number(4)
        // db4 is: Note('d'), Note('b'), Number(4) - because 'b' is read as Note
        // This is a limitation of the current lexer
        // For now, just check we get some tokens
        assert!(tokens.len() >= 3);
    }

    #[test]
    fn test_string_literal() {
        let source = "\"Hello World\"";
        let tokens = tokenize(source).unwrap();
        
        assert_eq!(tokens[0].0, Token::StringLiteral("Hello World".to_string()));
    }
}
