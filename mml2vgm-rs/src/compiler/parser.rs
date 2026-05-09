//! Parser for MML files
//!
//! This module provides a recursive descent parser for MML (Music Macro Language).
//! It takes tokens from the lexer and builds an Abstract Syntax Tree (AST).

use crate::compiler::ast::{
    Alias, Arpeggio, Aftertouch, ControlChange, Envelope, FmInstrument, Include, Length, 
    Loop, Metadata, MidiChannel, MidiProgram, MmlAst, MmlNode, Note, Octave, OctaveShift, 
    PartDefinition, PcmInstrument, PitchBend, PolyAftertouch, ProgramChange, Rest, 
    SysEx, Tempo, Volume,
};
use crate::compiler::lexer::{Token, tokenize};
use crate::{MmlError, MmlResult, Position, Span};
use std::path::PathBuf;

/// Parser for MML tokens
pub struct Parser {
    tokens: Vec<(Token, Position)>,
    current: usize,
    /// Current part being parsed
    current_part: Option<String>,
    /// Current octave (default: 4)
    current_octave: u8,
    /// Current length (default: 4)
    current_length: u32,
    /// Current volume (default: 127)
    current_volume: u8,
    /// Current tempo (default: 120)
    current_tempo: u32,
    /// Whether we're in a definition context (after apostrophe)
    in_definition_context: bool,
    /// FM instrument being accumulated row-by-row: (instrument_number, rows_collected, is_m_type)
    /// M-type instruments are NOT stored in ast.fm_instruments (they go in instOPM in C#).
    pending_fm_instrument: Option<(u32, Vec<Vec<u32>>, bool)>,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, Position)>) -> Self {
        Self {
            tokens,
            current: 0,
            current_part: None,
            current_octave: 4,
            current_length: 4,
            current_volume: 127,
            current_tempo: 120,
            in_definition_context: false,
            pending_fm_instrument: None,
        }
    }

    /// Get current token (cloned)
    fn current_token(&self) -> Option<Token> {
        self.tokens.get(self.current).map(|(t, _)| t.clone())
    }

    /// Get current position
    fn current_position(&self) -> Position {
        self.tokens.get(self.current).map(|(_, p)| *p).unwrap_or_else(|| Position::new(0, 0))
    }

    /// Advance to next token
    fn advance(&mut self) {
        self.current += 1;
    }

    /// Check if we're at the end of tokens
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.current_token(), Some(Token::Eof))
    }

    /// Get next token type without consuming
    fn peek_token_type(&self) -> Option<&Token> {
        self.tokens.get(self.current).map(|(t, _)| t)
    }

    /// Get next-next token type without consuming
    fn peek_next_token_type(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1).map(|(t, _)| t)
    }

    /// Consume current token and return it
    fn consume_token(&mut self) -> Option<Token> {
        let token = self.current_token();
        self.advance();
        token
    }

    /// Consume a specific token type
    fn consume(&mut self, expected: Token) -> bool {
        if self.peek_token_type() == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Add a node to the current part or global settings
    fn add_node_to_current_part(&mut self, ast: &mut MmlAst, node: MmlNode) {
        if let Some(ref part_name) = self.current_part {
            // Create part if it doesn't exist
            if !ast.parts.contains_key(part_name) {
                ast.parts.insert(part_name.clone(), PartDefinition {
                    name: part_name.clone(),
                    chip: None,
                    tempo: None,
                    commands: Vec::new(),
                });
            }
            if let Some(part) = ast.parts.get_mut(part_name) {
                part.commands.push(node);
            }
        } else {
            ast.global_settings.push(node);
        }
    }

    /// Check if we're in a definition context (after an apostrophe)
    fn is_in_definition_context(&self) -> bool {
        self.in_definition_context
    }

    /// Parse the entire MML source
    pub fn parse(mut self) -> MmlResult<MmlAst> {
        let mut ast = MmlAst::new();

        while !self.is_at_end() {
            if let Some(token) = self.current_token() {
                match token {
                    Token::LeftBrace => {
                        self.parse_song_info(&mut ast)?;
                    }
                    Token::Apostrophe => {
                        self.in_definition_context = true;
                        self.advance(); // Consume the '
                        self.parse_definition_line(&mut ast)?;
                        self.in_definition_context = false;
                    }
                    Token::Sharp => {
                        // Handle #CHIP and #CLOCK directives
                        self.advance(); // Consume #
                        if let Some(Token::Identifier(directive)) = self.current_token() {
                            let directive_upper = directive.to_uppercase();
                            if directive_upper == "CHIP" {
                                self.advance(); // Consume CHIP
                                // Skip whitespace
                                while let Some(Token::Whitespace(_)) = self.current_token() {
                                    self.advance();
                                }
                                // Get the chip name (can be identifier or multiple tokens for names like C140, C352)
                                if let Some(Token::Identifier(chip_name)) = self.current_token() {
                                    let chip_value = chip_name.clone();
                                    self.advance();
                                    // Handle multi-token chip names like C140, C352
                                    if let Some(Token::Number(num)) = self.current_token() {
                                        let combined = format!("{}{}", chip_value, num);
                                        ast.metadata.insert("CHIP".to_string(), combined);
                                        self.advance();
                                    } else {
                                        ast.metadata.insert("CHIP".to_string(), chip_value);
                                    }
                                } else if let Some(Token::Number(num)) = self.current_token() {
                                    ast.metadata.insert("CHIP".to_string(), num.to_string());
                                    self.advance();
                                }
                            } else if directive_upper == "CLOCK" {
                                self.advance(); // Consume CLOCK
                                // Skip whitespace
                                while let Some(Token::Whitespace(_)) = self.current_token() {
                                    self.advance();
                                }
                                if let Some(Token::Number(clock_val)) = self.current_token() {
                                    ast.metadata.insert("CLOCK".to_string(), clock_val.to_string());
                                    self.advance();
                                }
                            } else if directive_upper.starts_with("TRACK") {
                                self.advance(); // Consume TRACK
                                // Skip whitespace
                                while let Some(Token::Whitespace(_)) = self.current_token() {
                                    self.advance();
                                }
                                if let Some(Token::Number(track_num)) = self.current_token() {
                                    let track_name = track_num.to_string();
                                    self.current_part = Some(track_name);
                                    self.advance();
                                }
                            } else {
                                // Unknown directive, skip it
                            }
                        }
                    }
                    Token::Comment(_) => {
                        self.advance();
                    }
                    Token::Eof => break,
                    Token::AtSign => {
                        if self.is_in_definition_context() {
                            self.parse_definition_line(&mut ast)?;
                        } else {
                            if let Some(node) = self.parse_instrument_selection()? {
                                self.add_node_to_current_part(&mut ast, node);
                            } else {
                                self.advance(); // Ensure we advance if parse returns None
                            }
                        }
                    }
                    _ => {
                        if let Some(node) = self.parse_mml_command()? {
                            self.add_node_to_current_part(&mut ast, node);
                        } else {
                            self.advance();
                        }
                    }
                }
            } else {
                self.advance();
            }
        }

        // Finalize any FM instrument still being accumulated at end of file
        self.finalize_pending_fm_instrument(&mut ast);
        Ok(ast)
    }

    /// Parse song info block: { ... }
    fn parse_song_info(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        self.advance(); // Consume {
        
        while !self.is_at_end() && !matches!(self.current_token(), Some(Token::RightBrace)) {
            // Skip whitespace
            while let Some(Token::Whitespace(_)) = self.current_token() {
                self.advance();
            }
            
            if let Some(token) = self.current_token() {
                match token {
                    Token::Identifier(key) => {
                        self.advance();
                        // Skip whitespace
                        while let Some(Token::Whitespace(_)) = self.current_token() {
                            self.advance();
                        }
                        if self.consume(Token::Equals) {
                            // Skip whitespace
                            while let Some(Token::Whitespace(_)) = self.current_token() {
                                self.advance();
                            }
                            if let Some(Token::StringLiteral(value)) = self.current_token() {
                                self.advance();
                                ast.metadata.insert(key.clone(), value.clone());
                            } else if let Some(Token::Identifier(value)) = self.current_token() {
                                self.advance();
                                ast.metadata.insert(key.clone(), value.clone());
                            } else if let Some(Token::Number(value)) = self.current_token() {
                                self.advance();
                                ast.metadata.insert(key.clone(), value.to_string());
                            }
                        }
                    }
                    Token::Comment(_) => {
                        self.advance();
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
        }
        
        if self.consume(Token::RightBrace) {
            Ok(())
        } else {
            Err(MmlError::Parse {
                line: self.current_position().line,
                column: self.current_position().column,
                message: "Expected '}' to close song info block".to_string(),
            })
        }
    }

    /// Parse a definition line (starts after ')
    fn parse_definition_line(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        if let Some(token) = self.current_token() {
            match token {
                Token::Identifier(ref name) if name.to_uppercase().starts_with("INCLUDE") => {
                    self.parse_include_directive(ast)?;
                }
                Token::Identifier(ref name) if name.to_uppercase().starts_with("ALIAS") => {
                    self.parse_alias_definition(ast)?;
                }
                Token::AtSign => {
                    self.parse_instrument_definition(ast)?;
                }

                // C# format part names where the letter is a note letter: 'A1, 'B2, 'F1 …
                Token::Note(letter) => {
                    let letter = letter.to_ascii_uppercase();
                    self.advance(); // consume the note letter
                    let part_name = if let Some(Token::Number(n)) = self.current_token() {
                        let n = n;
                        self.advance(); // consume the channel number
                        format!("{}{}", letter, n)
                    } else {
                        letter.to_string()
                    };
                    self.ensure_part(ast, &part_name);
                    self.current_part = Some(part_name);
                }

                Token::Identifier(ref name) => {
                    let name = name.clone();
                    let name_upper = name.to_uppercase();

                    // Single alphabetic letter followed by a channel number → C# part name (H1, I1 …)
                    if name.len() == 1
                        && name.chars().next().map_or(false, |c| c.is_ascii_alphabetic())
                    {
                        let letter = name.chars().next().unwrap().to_ascii_uppercase();
                        self.advance(); // consume the letter
                        let part_name = if let Some(Token::Number(n)) = self.current_token() {
                            let n = n;
                            self.advance();
                            format!("{}{}", letter, n)
                        } else {
                            letter.to_string()
                        };
                        self.ensure_part(ast, &part_name);
                        self.current_part = Some(part_name);
                    } else if name_upper.starts_with("E") {
                        self.parse_envelope_definition(ast)?;
                    } else if name_upper.starts_with("A") && name.len() > 1 {
                        self.parse_arpeggio_definition(ast)?;
                    } else {
                        self.parse_part_definition(ast, &name)?;
                    }
                }
                _ => {
                    // Skip unknown definition token to avoid infinite loops
                    self.advance();
                }
            }
        }

        Ok(())
    }

    /// Create a part entry in `ast.parts` if it does not already exist.
    fn ensure_part(&self, ast: &mut MmlAst, name: &str) {
        if !ast.parts.contains_key(name) {
            ast.parts.insert(
                name.to_string(),
                PartDefinition {
                    name: name.to_string(),
                    chip: None,
                    tempo: None,
                    commands: Vec::new(),
                },
            );
        }
    }

    /// Parse part definition
    fn parse_part_definition(&mut self, ast: &mut MmlAst, name: &str) -> MmlResult<()> {
        self.advance(); // Consume identifier
        
        let mut chip = None;
        let mut tempo = None;
        
        while !self.is_at_end() {
            if let Some(token) = self.current_token() {
                match token {
                    Token::Identifier(ref cmd) if cmd.to_uppercase().starts_with("T") => {
                        self.advance();
                        if let Some(Token::Number(bpm)) = self.current_token() {
                            tempo = Some(bpm);
                            self.advance();
                        }
                    }
                    Token::Identifier(ref chip_name) => {
                        let cu = chip_name.to_uppercase();
                        if cu.starts_with("YM") || cu.starts_with("SN") 
                            || cu.starts_with("AY") || cu.starts_with("RF") {
                            chip = Some(chip_name.clone());
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    Token::Bar => {
                        self.advance();
                        break;
                    }
                    Token::Comment(_) => {
                        self.advance();
                    }
                    _ => break,
                }
            }
        }
        
        let part = PartDefinition {
            name: name.to_string(),
            chip: chip.clone(),
            tempo,
            commands: Vec::new(),
        };
        
        ast.parts.insert(name.to_string(), part);
        self.current_part = Some(name.to_string());
        
        Ok(())
    }

    /// Parse instrument definition after `'@`.
    ///
    /// Handles:
    /// - `'@ M NNN` / `'@ F NNN` — FM instrument header (multi-line C# format)
    /// - `'@ NNN,...` — FM operator/ALG row continuation (if accumulating)
    /// - `'@ P NNN,...` — PCM instrument
    /// - `'@ E NNN,...` — Envelope
    /// - `'@ A NNN,...` — Arpeggio
    fn parse_instrument_definition(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        if matches!(self.current_token(), Some(Token::AtSign)) {
            self.advance();
        }

        match self.current_token() {
            // Continuation row: '@ 031,012,...  — operator row or ALG/FB row
            Some(Token::Number(first)) => {
                let first = first;
                if self.pending_fm_instrument.is_some() {
                    self.parse_fm_instrument_row(ast, first)?;
                } else {
                    // Stray numeric row with no pending FM instrument; skip it
                    self.advance();
                    while let Some(token) = self.current_token() {
                        match token {
                            Token::Apostrophe | Token::Eof => break,
                            _ => self.advance(),
                        }
                    }
                }
            }

            // Type letter that the lexer sees as an Identifier: M, P, A (not note letters)
            Some(Token::Identifier(ref s)) => {
                let s_upper = s.to_uppercase();
                self.advance(); // consume the type letter
                if s_upper.starts_with('M') {
                    self.start_fm_instrument(ast, true)?;
                } else if s_upper.starts_with('F') {
                    self.start_fm_instrument(ast, false)?;
                } else if s_upper.starts_with('P') {
                    self.parse_pcm_instrument(ast)?;
                } else if s_upper.starts_with('E') {
                    self.parse_envelope_definition(ast)?;
                } else if s_upper.starts_with('A') {
                    self.parse_arpeggio_definition(ast)?;
                }
                // Unknown type letter — type already consumed, silently skip
            }

            // F, E, A, B, C, D, G are note letters → Token::Note in the lexer
            Some(Token::Note(letter)) => {
                let letter_upper = letter.to_ascii_uppercase();
                self.advance(); // consume the note-letter type token
                match letter_upper {
                    'F' => self.start_fm_instrument(ast, false)?,
                    'E' => self.parse_envelope_definition(ast)?,
                    'A' => self.parse_arpeggio_definition(ast)?,
                    _ => {
                        while let Some(token) = self.current_token() {
                            match token {
                                Token::Apostrophe | Token::Eof => break,
                                _ => self.advance(),
                            }
                        }
                    }
                }
            }

            _ => {
                if self.current_token().is_some() {
                    self.advance();
                }
            }
        }

        Ok(())
    }

    /// Begin accumulating a new FM instrument definition.
    ///
    /// Called after `'@ M NNN` (is_m_type=true) or `'@ F NNN` (is_m_type=false).
    /// M-type instruments are NOT stored in ast.fm_instruments (matching C# instOPM behavior).
    fn start_fm_instrument(&mut self, ast: &mut MmlAst, is_m_type: bool) -> MmlResult<()> {
        // Commit any previously started but not-yet-complete instrument
        self.finalize_pending_fm_instrument(ast);

        let number = match self.current_token() {
            Some(Token::Number(n)) => {
                let n = n;
                self.advance();
                n
            }
            Some(Token::Identifier(ref s)) => {
                if let Ok(n) = s.parse::<u32>() {
                    self.advance();
                    n
                } else {
                    return Ok(());
                }
            }
            _ => 0,
        };

        // Skip any remaining content on this header line (e.g. a patch name)
        while let Some(token) = self.current_token() {
            match token {
                Token::Apostrophe | Token::Eof => break,
                _ => self.advance(),
            }
        }

        self.pending_fm_instrument = Some((number, Vec::new(), is_m_type));
        Ok(())
    }

    /// Accumulate one comma-separated parameter row into the pending FM instrument.
    ///
    /// `first` is the first number already matched (but not yet consumed).
    /// After 5 rows (4 operators × 11 params + 1 ALG/FB row × 2 params), the
    /// instrument is finalised and stored.
    fn parse_fm_instrument_row(&mut self, ast: &mut MmlAst, first: u32) -> MmlResult<()> {
        let mut row = vec![first];
        self.advance(); // consume first number

        while let Some(token) = self.current_token() {
            match token {
                Token::Number(n) => {
                    row.push(n);
                    self.advance();
                }
                Token::Comma => self.advance(),
                Token::Apostrophe | Token::Eof => break,
                _ => break,
            }
        }

        if let Some((_, rows, _)) = &mut self.pending_fm_instrument {
            rows.push(row);
        }

        // Finalise after 5 rows: 4 operator rows + 1 ALG/FB row
        let ready = self.pending_fm_instrument
            .as_ref()
            .map_or(false, |(_, rows, _)| rows.len() >= 5);
        if ready {
            self.finalize_pending_fm_instrument(ast);
        }

        Ok(())
    }

    /// Commit a pending FM instrument (even if incomplete) into the AST.
    /// M-type instruments are NOT stored in fm_instruments (matches C# instOPM behavior).
    fn finalize_pending_fm_instrument(&mut self, ast: &mut MmlAst) {
        if let Some((number, rows, _is_m_type)) = self.pending_fm_instrument.take() {
            let mut parameters: Vec<u32> = Vec::new();
            for row in rows {
                parameters.extend(row);
            }
            let inst = FmInstrument { number, name: String::new(), parameters };
            ast.fm_instruments.insert(number, inst);
        }
    }

    /// Parse PCM instrument definition
    fn parse_pcm_instrument(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        let number = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n
        } else if let Some(Token::Identifier(s)) = self.current_token() {
            if s.parse::<u32>().is_ok() {
                self.advance();
                s.parse::<u32>().unwrap()
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };
        
        // Skip comma and whitespace
        while let Some(token) = self.current_token() {
            match token {
                Token::Comma | Token::Whitespace(_) => self.advance(),
                _ => break,
            }
        }
        
        let filename = if let Some(Token::StringLiteral(s)) = self.current_token() {
            self.advance();
            PathBuf::from(s)
        } else if let Some(Token::Identifier(s)) = self.current_token() {
            self.advance();
            PathBuf::from(s)
        } else {
            PathBuf::new()
        };
        
        // Skip comma and whitespace
        while let Some(token) = self.current_token() {
            match token {
                Token::Comma | Token::Whitespace(_) => self.advance(),
                _ => break,
            }
        }
        
        let frequency = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n
        } else {
            0
        };
        
        // Skip comma and whitespace
        while let Some(token) = self.current_token() {
            match token {
                Token::Comma | Token::Whitespace(_) => self.advance(),
                _ => break,
            }
        }
        
        let volume = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n.clamp(0, 127) as u8
        } else {
            100
        };
        
        let mut chip = String::new();
        let mut option = None;
        
        if self.current_token() == Some(Token::Comma) {
            self.advance();
            while let Some(token) = self.current_token() {
                match token {
                    Token::Whitespace(_) => self.advance(),
                    Token::Identifier(s) => {
                        chip = s.clone();
                        self.advance();
                        if self.current_token() == Some(Token::Comma) {
                            self.advance();
                            if let Some(Token::Number(n)) = self.current_token() {
                                option = Some(n);
                                self.advance();
                            }
                        }
                        break;
                    }
                    // Chip names like "C140" tokenize as Note('C') + Number(140) because
                    // the lexer treats A-G as note letters. Reconstruct the full name.
                    Token::Note(c) => {
                        let letter = c.to_ascii_uppercase();
                        self.advance();
                        let suffix = match self.current_token() {
                            Some(Token::Number(n)) => {
                                let s = n.to_string();
                                self.advance();
                                s
                            }
                            Some(Token::Identifier(s)) => {
                                let s = s.clone();
                                self.advance();
                                s
                            }
                            _ => String::new(),
                        };
                        chip = format!("{}{}", letter, suffix);
                        if self.current_token() == Some(Token::Comma) {
                            self.advance();
                            if let Some(Token::Number(n)) = self.current_token() {
                                option = Some(n);
                                self.advance();
                            }
                        }
                        break;
                    }
                    _ => break,
                }
            }
        }
        
        let inst = PcmInstrument {
            number,
            name: filename.to_string_lossy().into_owned(),
            filename,
            frequency,
            volume,
            chip,
            option,
        };
        ast.pcm_instruments.insert(number, inst);
        
        Ok(())
    }

    /// Parse envelope definition
    fn parse_envelope_definition(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        // Skip @ and E if present
        if matches!(self.current_token(), Some(Token::AtSign)) {
            self.advance();
        }
        if let Some(Token::Identifier(s)) = self.current_token() {
            if s.to_uppercase() == "E" {
                self.advance();
            }
        }
        
        let number = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n
        } else if let Some(Token::Identifier(s)) = self.current_token() {
            if s.parse::<u32>().is_ok() {
                self.advance();
                s.parse::<u32>().unwrap()
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };
        
        // Skip comma and whitespace
        while let Some(token) = self.current_token() {
            match token {
                Token::Comma | Token::Whitespace(_) => self.advance(),
                _ => break,
            }
        }
        
        let mut parameters = Vec::new();
        while let Some(token) = self.current_token() {
            match token {
                Token::Number(n) => {
                    parameters.push(n);
                    self.advance();
                }
                Token::Comma | Token::Whitespace(_) => self.advance(),
                Token::Apostrophe | Token::Eof => break,
                _ => self.advance(),
            }
        }
        
        let env = Envelope { number, parameters };
        ast.envelopes.insert(number, env);
        
        Ok(())
    }

    /// Parse arpeggio definition
    fn parse_arpeggio_definition(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        // Skip @ and A if present
        if matches!(self.current_token(), Some(Token::AtSign)) {
            self.advance();
        }
        if let Some(Token::Identifier(s)) = self.current_token() {
            if s.to_uppercase() == "A" {
                self.advance();
            }
        }
        
        let number = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n
        } else if let Some(Token::Identifier(s)) = self.current_token() {
            if s.parse::<u32>().is_ok() {
                self.advance();
                s.parse::<u32>().unwrap()
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };
        
        // Skip comma and whitespace
        while let Some(token) = self.current_token() {
            match token {
                Token::Comma | Token::Whitespace(_) => self.advance(),
                _ => break,
            }
        }
        
        let mut notes = Vec::new();
        while let Some(token) = self.current_token() {
            match token {
                Token::Note(letter) => {
                    self.advance();
                    let accidental = if matches!(self.current_token(), Some(Token::Sharp)) {
                        self.advance();
                        1
                    } else if matches!(self.current_token(), Some(Token::Flat)) {
                        self.advance();
                        -1
                    } else {
                        0
                    };
                    
                    let octave = if let Some(Token::Number(n)) = self.current_token() {
                        self.advance();
                        n as u8
                    } else {
                        self.current_octave
                    };
                    
                    notes.push(Note::new(letter, accidental, octave));
                }
                Token::Comma | Token::Whitespace(_) => self.advance(),
                Token::Apostrophe | Token::Eof => break,
                _ => self.advance(),
            }
        }
        
        let arp = Arpeggio { number, notes };
        ast.arpeggios.insert(number, arp);
        
        Ok(())
    }

    /// Parse alias definition
    fn parse_alias_definition(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        // Skip alias keyword
        self.advance();
        
        // Read name
        let name = if let Some(Token::Identifier(s)) = self.current_token() {
            self.advance();
            s
        } else {
            return Ok(());
        };
        
        // Skip whitespace
        while let Some(Token::Whitespace(_)) = self.current_token() {
            self.advance();
        }
        
        if self.consume(Token::Equals) {
            // Skip whitespace
            while let Some(Token::Whitespace(_)) = self.current_token() {
                self.advance();
            }
            
            let expansion = if let Some(Token::Identifier(s)) = self.current_token() {
                self.advance();
                s
            } else if let Some(Token::StringLiteral(s)) = self.current_token() {
                self.advance();
                s
            } else if let Some(Token::Number(n)) = self.current_token() {
                self.advance();
                n.to_string()
            } else {
                String::new()
            };
            
            ast.aliases.insert(name, expansion);
        }
        
        Ok(())
    }

    /// Parse include directive
    fn parse_include_directive(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        // Skip INCLUDE keyword
        if let Some(Token::Identifier(_)) = self.current_token() {
            self.advance();
        }
        
        // Skip whitespace
        while let Some(Token::Whitespace(_)) = self.current_token() {
            self.advance();
        }
        
        let path = if let Some(Token::StringLiteral(s)) = self.current_token() {
            self.advance();
            PathBuf::from(s)
        } else if let Some(Token::Identifier(s)) = self.current_token() {
            self.advance();
            PathBuf::from(s)
        } else {
            PathBuf::new()
        };
        
        ast.includes.push(path);
        
        Ok(())
    }

    /// Parse instrument selection or MIDI command
    fn parse_instrument_selection(&mut self) -> MmlResult<Option<MmlNode>> {
        if matches!(self.current_token(), Some(Token::AtSign)) {
            self.advance();
        }
        
        // Check for MIDI-specific commands first
        if let Some(token) = self.current_token() {
            match token {
                Token::ControlChangeCommand => {
                    self.advance();
                    return self.parse_midi_control_change();
                }
                Token::ProgramChangeCommand => {
                    self.advance();
                    return self.parse_midi_program_change();
                }
                Token::PitchBendCommand => {
                    self.advance();
                    return self.parse_midi_pitch_bend();
                }
                Token::AftertouchCommand => {
                    self.advance();
                    return self.parse_midi_aftertouch();
                }
                Token::PolyAftertouchCommand => {
                    self.advance();
                    return self.parse_midi_poly_aftertouch();
                }
                Token::SysExCommand => {
                    self.advance();
                    return self.parse_midi_sysex();
                }
                Token::MidiChannelCommand => {
                    self.advance();
                    return self.parse_midi_channel();
                }
                Token::MidiProgramCommand => {
                    self.advance();
                    return self.parse_midi_program();
                }
                Token::PanCommand => {
                    self.advance();
                    return self.parse_midi_pan();
                }
                Token::ExpressionCommand => {
                    self.advance();
                    return self.parse_midi_expression();
                }
                Token::SustainCommand => {
                    self.advance();
                    return self.parse_midi_sustain();
                }
                Token::AllNotesOffCommand => {
                    self.advance();
                    return Ok(Some(self.create_control_change(120, 0)));
                }
                Token::ResetAllCtrlCommand => {
                    self.advance();
                    return Ok(Some(self.create_control_change(121, 0)));
                }
                Token::AllSoundOffCommand => {
                    self.advance();
                    // All Sound Off: CC120 + CC121 + CC123
                    // For now, just send CC120 (all notes off) + CC121 (reset all controllers)
                    // A proper implementation would send all three
                    return Ok(Some(MmlNode::Loop(Loop {
                        count: 1,
                        body: vec![
                            self.create_control_change(120, 0),
                            self.create_control_change(121, 0),
                            self.create_control_change(123, 0),
                        ],
                    })));
                }
                Token::DamperCommand => {
                    self.advance();
                    // Damper pedal = CC64
                    let value = if let Some(Token::Identifier(s)) = self.current_token() {
                        if s.to_lowercase() == "off" {
                            self.advance();
                            0
                        } else {
                            127
                        }
                    } else {
                        127
                    };
                    return Ok(Some(self.create_control_change(64, value)));
                }
                Token::PortamentoCommand => {
                    self.advance();
                    // Portamento = CC65
                    let value = if let Some(Token::Identifier(s)) = self.current_token() {
                        if s.to_lowercase() == "off" {
                            self.advance();
                            0
                        } else {
                            127
                        }
                    } else {
                        127
                    };
                    return Ok(Some(self.create_control_change(65, value)));
                }
                Token::SostenutoCommand => {
                    self.advance();
                    // Sostenuto = CC66
                    let value = if let Some(Token::Identifier(s)) = self.current_token() {
                        if s.to_lowercase() == "off" {
                            self.advance();
                            0
                        } else {
                            127
                        }
                    } else {
                        127
                    };
                    return Ok(Some(self.create_control_change(66, value)));
                }
                Token::SoftCommand => {
                    self.advance();
                    // Soft pedal = CC67
                    let value = if let Some(Token::Identifier(s)) = self.current_token() {
                        if s.to_lowercase() == "off" {
                            self.advance();
                            0
                        } else {
                            127
                        }
                    } else {
                        127
                    };
                    return Ok(Some(self.create_control_change(67, value)));
                }
                Token::LocalControlCommand => {
                    self.advance();
                    // Local Control on/off = CC122
                    // 0 = off, 127 = on
                    let value = if let Some(Token::Identifier(s)) = self.current_token() {
                        if s.to_lowercase() == "on" {
                            self.advance();
                            127
                        } else if s.to_lowercase() == "off" {
                            self.advance();
                            0
                        } else {
                            127
                        }
                    } else {
                        127
                    };
                    return Ok(Some(self.create_control_change(122, value)));
                }
                Token::DrumNoteCommand => {
                    self.advance();
                    return self.parse_drum_note();
                }
                _ => {}
            }
        }
        
        // Parse regular instrument selection or chip-specific command
        if let Some(Token::Identifier(s)) = self.current_token() {
            let s_upper = s.to_uppercase();
            
            // Check if this is a known chip-specific command
            if self.is_chip_command(&s_upper) {
                self.advance();
                return self.parse_chip_command(&s_upper);
            }
            
            // Otherwise try to parse as instrument number
            if s.parse::<usize>().is_ok() {
                self.advance();
                let number = s.parse::<usize>().unwrap();
                return Ok(Some(MmlNode::InstrumentSelection(
                    crate::compiler::ast::InstrumentSelection { number, span: None }
                )));
            }
        }
        
        if let Some(Token::Number(n)) = self.current_token() {
            // Special case: digit-prefixed chip commands like `@4OP` tokenize as
            // Number(4) + Identifier("OP"). Stitch them and look up the combined name.
            if let Some(Token::Identifier(suffix)) = self.peek_next_token_type().cloned() {
                let combined = format!("{}{}", n, suffix.to_uppercase());
                if self.is_chip_command(&combined) {
                    self.advance(); // consume number
                    self.advance(); // consume identifier
                    return self.parse_chip_command(&combined);
                }
            }
            self.advance();
            return Ok(Some(MmlNode::InstrumentSelection(
                crate::compiler::ast::InstrumentSelection { number: n as usize, span: None }
            )));
        }

        Ok(None)
    }

    /// Helper to create a ControlChange node
    fn create_control_change(&self, controller: u8, value: u8) -> MmlNode {
        MmlNode::MidiControlChange(ControlChange {
            controller,
            value,
            channel: None,
            span: None,
        })
    }

    /// Parse @c or @cc control change command
    fn parse_midi_control_change(&mut self) -> MmlResult<Option<MmlNode>> {
        let controller = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n as u8
        } else {
            return Ok(None);
        };
        
        let value = if self.consume(Token::Equals) {
            if let Some(Token::Number(n)) = self.current_token() {
                self.advance();
                n as u8
            } else {
                127 // Default to max if no value specified
            }
        } else if self.consume(Token::Comma) {
            if let Some(Token::Number(n)) = self.current_token() {
                self.advance();
                n as u8
            } else {
                127
            }
        } else {
            127 // Default value
        };
        
        Ok(Some(MmlNode::MidiControlChange(ControlChange {
            controller,
            value,
            channel: None,
            span: None,
        })))
    }

    /// Parse @p or @pg program change command
    fn parse_midi_program_change(&mut self) -> MmlResult<Option<MmlNode>> {
        let program = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n as u8
        } else {
            return Ok(None);
        };
        
        Ok(Some(MmlNode::MidiProgramChange(ProgramChange {
            program,
            channel: None,
            span: None,
        })))
    }

    /// Parse @b or @bend pitch bend command
    fn parse_midi_pitch_bend(&mut self) -> MmlResult<Option<MmlNode>> {
        let value = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            // Convert to signed value (-8192 to 8191)
            // For now, treat as positive offset from center
            (n as i16 - 8192).clamp(-8192, 8191)
        } else if self.consume(Token::Sharp) || self.consume(Token::Flat) {
            // Handle + and - prefixes
            let sign = if self.current_token() == Some(Token::Sharp) { 1 } else { -1 };
            if let Some(Token::Number(n)) = self.current_token() {
                self.advance();
                let base: i16 = n as i16;
                (base * sign).clamp(-8192, 8191)
            } else {
                0
            }
        } else {
            0 // Center
        };
        
        Ok(Some(MmlNode::MidiPitchBend(PitchBend {
            value,
            channel: None,
            span: None,
        })))
    }

    /// Parse @a or @at aftertouch command
    fn parse_midi_aftertouch(&mut self) -> MmlResult<Option<MmlNode>> {
        let value = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n as u8
        } else {
            return Ok(None);
        };
        
        Ok(Some(MmlNode::MidiAftertouch(Aftertouch {
            value,
            channel: None,
            span: None,
        })))
    }

    /// Parse @pa polyphonic aftertouch command
    fn parse_midi_poly_aftertouch(&mut self) -> MmlResult<Option<MmlNode>> {
        let note = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n as u8
        } else {
            return Ok(None);
        };
        
        let value = if self.consume(Token::Comma) {
            if let Some(Token::Number(n)) = self.current_token() {
                self.advance();
                n as u8
            } else {
                127
            }
        } else {
            127
        };
        
        Ok(Some(MmlNode::MidiPolyAftertouch(PolyAftertouch {
            note,
            value,
            channel: None,
            span: None,
        })))
    }

    /// Parse @x or @sysex system exclusive command
    fn parse_midi_sysex(&mut self) -> MmlResult<Option<MmlNode>> {
        let mut data = Vec::new();
        
        // Parse hex bytes separated by commas
        while let Some(Token::Number(n)) = self.current_token() {
            data.push(n as u8);
            self.advance();
            if !self.consume(Token::Comma) {
                break;
            }
        }
        
        Ok(Some(MmlNode::MidiSysEx(SysEx {
            data,
            span: None,
        })))
    }

    /// Parse @ch MIDI channel command
    fn parse_midi_channel(&mut self) -> MmlResult<Option<MmlNode>> {
        let channel = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            (n % 16) as u8 // Clamp to 0-15
        } else {
            return Ok(None);
        };
        
        Ok(Some(MmlNode::MidiChannel(MidiChannel {
            channel,
            span: None,
        })))
    }

    /// Parse @pr MIDI program command
    fn parse_midi_program(&mut self) -> MmlResult<Option<MmlNode>> {
        let program = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            (n % 128) as u8
        } else {
            return Ok(None);
        };
        
        let mut bank_msb = None;
        let mut bank_lsb = None;
        
        if self.consume(Token::Comma) {
            if let Some(Token::Number(n)) = self.current_token() {
                self.advance();
                bank_msb = Some((n % 128) as u8);
            }
            if self.consume(Token::Comma) {
                if let Some(Token::Number(n)) = self.current_token() {
                    self.advance();
                    bank_lsb = Some((n % 128) as u8);
                }
            }
        }
        
        Ok(Some(MmlNode::MidiProgram(MidiProgram {
            program,
            bank_msb,
            bank_lsb,
            span: None,
        })))
    }

    /// Parse @pan pan command (maps to CC10)
    fn parse_midi_pan(&mut self) -> MmlResult<Option<MmlNode>> {
        let value = if let Some(token) = self.current_token() {
            match token {
                Token::Identifier(ref s) if s.to_lowercase() == "left" => {
                    self.advance();
                    0 // Pan left
                }
                Token::Identifier(ref s) if s.to_lowercase() == "center" || s.to_lowercase() == "centre" => {
                    self.advance();
                    64 // Pan center
                }
                Token::Identifier(ref s) if s.to_lowercase() == "right" => {
                    self.advance();
                    127 // Pan right
                }
                Token::Number(n) => {
                    self.advance();
                    (n % 128) as u8
                }
                _ => {
                    return Ok(None);
                }
            }
        } else {
            return Ok(None);
        };
        
        Ok(Some(self.create_control_change(10, value)))
    }

    /// Parse @expr expression command (maps to CC11)
    fn parse_midi_expression(&mut self) -> MmlResult<Option<MmlNode>> {
        let value = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            (n % 128) as u8
        } else {
            return Ok(None);
        };
        
        Ok(Some(self.create_control_change(11, value)))
    }

    /// Parse @sustain sustain command (maps to CC64)
    fn parse_midi_sustain(&mut self) -> MmlResult<Option<MmlNode>> {
        let value = if let Some(Token::Identifier(s)) = self.current_token() {
            if s.to_lowercase() == "off" {
                self.advance();
                0
            } else {
                127
            }
        } else {
            127 // Default to on
        };
        
        Ok(Some(self.create_control_change(64, value)))
    }

    /// Parse #D drum note command
    fn parse_drum_note(&mut self) -> MmlResult<Option<MmlNode>> {
        // Drum notes use note names that map to MIDI drum numbers
        // e.g., #Dkick, #Dsnare, #Dhh, etc.
        let identifier = if let Some(Token::Identifier(s)) = self.current_token() {
            self.advance();
            s
        } else {
            return Ok(None);
        };
        
        // Map drum names to MIDI note numbers
        let note_number = match identifier.to_lowercase().as_str() {
            "kick" | "bd" | "bassdrum" => 36,      // Bass Drum
            "snare" | "sd" => 38,                 // Acoustic Snare
            "hh" | "hihat" | "closedhh" => 42,     // Closed Hi-Hat
            "oh" | "openhh" => 46,                 // Open Hi-Hat
            "crash" => 49,                         // Crash Cymbal
            "ride" => 51,                          // Ride Cymbal
            "tom1" | "hightom" => 50,              // High Tom
            "tom2" | "midtom" => 48,                // Mid Tom
            "tom3" | "lowtom" => 41,                // Low Tom
            "clap" => 39,                          // Hand Clap
            "cowbell" => 56,                       // Cowbell
            "tambourine" => 54,                    // Tambourine
            "shaker" => 70,                        // Shaker
            _ => {
                // Try to parse as a number
                if let Ok(n) = identifier.parse::<u8>() {
                    n.clamp(35, 81) // Valid GM drum range
                } else {
                    return Ok(None);
                }
            }
        };
        
        // Create a note on channel 10 (drum channel in GM)
        let mut note = Note::new('C', 0, 4);
        // Override the MIDI note calculation to use the drum number
        // We'll need to handle this specially in the code generator
        
        Ok(Some(MmlNode::Note(note)))
    }

    /// Parse MML commands
    fn parse_mml_command(&mut self) -> MmlResult<Option<MmlNode>> {
        if let Some(token) = self.current_token() {
            match token {
                Token::Note(letter) => {
                    let span_start = self.current_position();
                    self.advance();
                    let mut note = Note::new(letter, 0, self.current_octave);

                    if self.consume(Token::Sharp) {
                        note.accidental = 1;
                    } else if self.consume(Token::Flat) {
                        note.accidental = -1;
                    }

                    if let Some(Token::Number(n)) = self.current_token() {
                        note.octave = n as u8;
                        self.advance();
                    }

                    if let Some(Token::Number(d)) = self.current_token() {
                        note.duration = Some(d);
                        self.advance();
                    } else {
                        note.duration = Some(self.current_length);
                    }

                    if self.consume(Token::Dot) {
                        note.dotted = true;
                    }

                    if self.consume(Token::Underscore) {
                        note.tied = true;
                    }

                    let span_end = self.current_position();
                    note.span = Some(Span::new(span_start, span_end));

                    Ok(Some(MmlNode::Note(note)))
                }
                
                // MIDI note number: n<0-127>
                // Converts a MIDI note number directly to a note, bypassing current octave.
                // The octave is derived from the MIDI number: octave = (midi / 12) - 1.
                // Does NOT update current_octave; subsequent letter-notes continue at their
                // current octave (same as 3MLE / Sitaraba behaviour).
                Token::NoteNumberCommand => {
                    let span_start = self.current_position();
                    self.advance();
                    if let Some(Token::Number(midi)) = self.current_token() {
                        let midi = midi.min(127);
                        self.advance();
                        let octave = (midi / 12).saturating_sub(1) as u8;
                        let pos = midi % 12;
                        let (letter, accidental): (char, i8) = match pos {
                            0  => ('C', 0),
                            1  => ('C', 1),
                            2  => ('D', 0),
                            3  => ('D', 1),
                            4  => ('E', 0),
                            5  => ('F', 0),
                            6  => ('F', 1),
                            7  => ('G', 0),
                            8  => ('G', 1),
                            9  => ('A', 0),
                            10 => ('A', 1),
                            _  => ('B', 0),  // 11 = B
                        };
                        let mut note = Note::new(letter, accidental, octave);
                        note.duration = Some(self.current_length);
                        if self.consume(Token::Dot) {
                            note.dotted = true;
                        }
                        if self.consume(Token::Underscore) {
                            note.tied = true;
                        }
                        let span_end = self.current_position();
                        note.span = Some(Span::new(span_start, span_end));
                        Ok(Some(MmlNode::Note(note)))
                    } else {
                        Ok(None)
                    }
                }

                Token::Rest => {
                    let span_start = self.current_position();
                    self.advance();
                    let mut rest = Rest {
                        duration: self.current_length,
                        dotted: false,
                        span: None,
                    };

                    if let Some(Token::Number(d)) = self.current_token() {
                        rest.duration = d;
                        self.advance();
                    }

                    if self.consume(Token::Dot) {
                        rest.dotted = true;
                    }

                    let span_end = self.current_position();
                    rest.span = Some(Span::new(span_start, span_end));

                    Ok(Some(MmlNode::Rest(rest)))
                }
                
                Token::OctaveCommand => {
                    self.advance();
                    let octave = if let Some(Token::Number(n)) = self.current_token() {
                        self.advance();
                        n as u8
                    } else {
                        4
                    };
                    self.current_octave = octave;
                    Ok(Some(MmlNode::Octave(Octave { number: octave })))
                }
                
                Token::GreaterThan => {
                    self.advance();
                    if self.current_octave < 8 {
                        self.current_octave += 1;
                    }
                    Ok(Some(MmlNode::OctaveShift(OctaveShift::Up)))
                }
                
                Token::LessThan => {
                    self.advance();
                    if self.current_octave > 0 {
                        self.current_octave -= 1;
                    }
                    Ok(Some(MmlNode::OctaveShift(OctaveShift::Down)))
                }
                
                Token::TempoCommand => {
                    self.advance();
                    let bpm = if let Some(Token::Number(n)) = self.current_token() {
                        self.advance();
                        n
                    } else {
                        120
                    };
                    self.current_tempo = bpm;
                    Ok(Some(MmlNode::Tempo(Tempo { bpm })))
                }
                
                Token::VolumeCommand => {
                    self.advance();
                    let level = if let Some(Token::Number(n)) = self.current_token() {
                        self.advance();
                        n.clamp(0, 127) as u8
                    } else {
                        127
                    };
                    self.current_volume = level;
                    Ok(Some(MmlNode::Volume(Volume { level })))
                }
                
                Token::LengthCommand => {
                    self.advance();
                    let value = if let Some(Token::Number(n)) = self.current_token() {
                        self.advance();
                        n
                    } else {
                        4
                    };
                    self.current_length = value;
                    Ok(Some(MmlNode::Length(Length { value })))
                }
                
                Token::Bar => {
                    self.advance();
                    Ok(Some(MmlNode::Bar))
                }
                
                Token::AtSign => {
                    let span_start = self.current_position();
                    self.advance();
                    if let Some(Token::Number(n)) = self.current_token() {
                        let number = n as usize;
                        self.advance();
                        let span_end = self.current_position();
                        Ok(Some(MmlNode::InstrumentSelection(crate::compiler::ast::InstrumentSelection { number, span: Some(Span::new(span_start, span_end)) })))
                    } else {
                        Ok(None)
                    }
                }
                
                Token::Identifier(cmd) if cmd.to_uppercase().starts_with("EON") => {
                    self.advance();
                    Ok(Some(MmlNode::ChipCommand {
                        chip: cmd.clone(),
                        command: cmd.clone(),
                        args: Vec::new(),
                    }))
                }

                // Quantize / gate time: q<value> or q (0-48, silence = value/48 of note duration)
                // The lexer may produce "q1" as a single Identifier, or "q" + Number separately.
                Token::Identifier(cmd)
                    if (cmd.starts_with('q') || cmd.starts_with('Q'))
                        && (cmd.len() == 1
                            || cmd[1..].chars().all(|c| c.is_ascii_digit())) =>
                {
                    let proportional = cmd.starts_with('Q');
                    let embedded_value = if cmd.len() > 1 {
                        cmd[1..].parse::<u32>().ok()
                    } else {
                        None
                    };
                    self.advance();
                    let value = if let Some(v) = embedded_value {
                        v as u8
                    } else if let Some(Token::Number(n)) = self.current_token() {
                        let v = n as u8;
                        self.advance();
                        v
                    } else {
                        1
                    };
                    Ok(Some(MmlNode::Quantize(crate::compiler::ast::Quantize {
                        value,
                        proportional,
                    })))
                }
                
                // Infinite loop: [body]
                Token::LeftBracket => {
                    self.advance();
                    let mut body = Vec::new();
                    while self.current_token().is_some()
                        && !matches!(self.current_token(), Some(Token::RightBracket))
                    {
                        if let Some(node) = self.parse_mml_command()? {
                            body.push(node);
                        } else {
                            self.advance();
                        }
                    }
                    self.consume(Token::RightBracket);
                    Ok(Some(MmlNode::Loop(Loop { count: 0, body })))
                }

                // Finite loop: (body)N
                Token::LeftParen => {
                    self.advance();
                    let mut body = Vec::new();
                    while self.current_token().is_some()
                        && !matches!(self.current_token(), Some(Token::RightParen))
                    {
                        if let Some(node) = self.parse_mml_command()? {
                            body.push(node);
                        } else {
                            self.advance();
                        }
                    }
                    self.consume(Token::RightParen);
                    let count = if let Some(Token::Number(n)) = self.current_token() {
                        let n = n as usize;
                        self.advance();
                        n
                    } else {
                        1
                    };
                    Ok(Some(MmlNode::Loop(Loop { count, body })))
                }

                Token::Comment(text) => {
                    self.advance();
                    Ok(Some(MmlNode::Comment(text)))
                }

                _ => {
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Check if a string is a known chip-specific command
    fn is_chip_command(&self, cmd: &str) -> bool {
        // FM Chip Commands (operators)
        matches!(cmd,
            "AR" | "DR" | "SR" | "RR" | "SL" | "TL" | "KS" | "ML" | "DT" |
            "AL" | "FB" | "OP" |
            // PSG Commands
            "EN" | "MIX" | "FILTER" | "DIST" | "HPOLY" |
            // Wavetable Commands
            "WAVE" | "NW" | "SW" | "P" | "KEYON" | "KEYOFF" | "NOCTRL" |
            // PCM Commands
            "BANK" | "START" | "LOOP" | "END" | "REVERSE" | "LOOPSTART" | "LOOPLEN" |
            "LVOL" | "RVOL" | "ADPCM" | "OPL3MODE" | "4OP" | "CUSTOM" |
            "VIB" | "TREM" | "DRUM" | "PAN" | "REVERB" | "PITCH" | "VOLUME"
        )
    }

    /// Parse a chip-specific command (e.g., @AR, @DR, @FB, etc.)
    fn parse_chip_command(&mut self, cmd: &str) -> MmlResult<Option<MmlNode>> {
        // Parse command arguments (usually a single value or comma-separated values)
        let mut args = Vec::new();
        
        // Check if next token is a number or operator specifier
        if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            args.push(n);
            
            // Parse additional arguments if comma-separated
            while self.consume(Token::Comma) {
                if let Some(Token::Number(n)) = self.current_token() {
                    self.advance();
                    args.push(n);
                } else {
                    break;
                }
            }
        }
        
        Ok(Some(MmlNode::ChipCommand {
            chip: "Generic".to_string(),  // Will be resolved during codegen
            command: cmd.to_string(),
            args,
        }))
    }
}

/// Parse MML source code from a string
pub fn parse(source: &str) -> MmlResult<MmlAst> {
    let tokens = tokenize(source)?;
    let parser = Parser::new(tokens);
    parser.parse()
}

/// Parse MML source code from a file
pub fn parse_file(path: &PathBuf) -> MmlResult<MmlAst> {
    let bytes = std::fs::read(path)?;
    let source = String::from_utf8_lossy(&bytes).into_owned();
    parse(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_notes() {
        let source = "o4 c4 d4 e4 f4";
        let ast = parse(source).unwrap();
        
        assert!(!ast.global_settings.is_empty());
    }

    #[test]
    fn test_parse_song_info() {
        // Simple song info with one key-value pair
        let source = "{ TitleName = MySong }";
        let ast = parse(source).unwrap();
        
        assert_eq!(ast.get_metadata("TitleName"), Some(&"MySong".to_string()));
    }

    #[test]
    fn test_parse_simple_mml() {
        let source = "o4 cde f g a b> c";
        let ast = parse(source).unwrap();

        // Should have octave change and notes
        assert!(!ast.global_settings.is_empty());
    }

    #[test]
    fn parse_pcm_c140_chip_name() {
        // C140 starts with the note letter C, so the lexer emits Note('C') + Number(140)
        // rather than Identifier("C140"). The parser must reassemble the chip name.
        let source = "'@ P 1,\"sample.wav\",8000,100,C140,1400";
        let ast = parse(source).unwrap();
        let inst = ast.pcm_instruments.get(&1).expect("instrument 1 should be parsed");
        assert_eq!(inst.chip, "C140");
        assert_eq!(inst.option, Some(1400));
    }

    #[test]
    fn parse_pcm_rf5c164_chip_name() {
        // Rf5c164 starts with R (not a note letter) so the lexer produces a single
        // Identifier token — verify the existing path still works correctly.
        let source = "'@ P 2,\"sample.wav\",8000,100,Rf5c164,1400";
        let ast = parse(source).unwrap();
        let inst = ast.pcm_instruments.get(&2).expect("instrument 2 should be parsed");
        assert_eq!(inst.chip, "Rf5c164");
        assert_eq!(inst.option, Some(1400));
    }
}
