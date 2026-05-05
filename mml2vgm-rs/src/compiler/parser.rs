//! Parser for MML files
//!
//! This module provides a recursive descent parser for MML (Music Macro Language).
//! It takes tokens from the lexer and builds an Abstract Syntax Tree (AST).

use crate::compiler::ast::{
    Alias, Arpeggio, Envelope, FmInstrument, Include, Length, Loop, Metadata, MmlAst, MmlNode, 
    Note, Octave, OctaveShift, PartDefinition, PcmInstrument, Rest, Tempo, Volume,
};
use crate::compiler::lexer::{Token, tokenize};
use crate::{MmlError, MmlResult, Position};
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
                Token::Identifier(ref name) => {
                    let name_upper = name.to_uppercase();
                    if name_upper.starts_with("E") {
                        self.parse_envelope_definition(ast)?;
                    } else if name_upper.starts_with("A") && name.len() > 1 {
                        self.parse_arpeggio_definition(ast)?;
                    } else {
                        self.parse_part_definition(ast, name)?;
                    }
                }
                _ => {
                    // Skip unknown definition
                }
            }
        }
        
        Ok(())
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

    /// Parse instrument definition
    fn parse_instrument_definition(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
        // Consume @ if present
        if matches!(self.current_token(), Some(Token::AtSign)) {
            self.advance();
        }
        
        let type_char = if let Some(Token::Identifier(s)) = self.current_token() {
            let s_upper = s.to_uppercase();
            self.advance();
            if s_upper.starts_with("F") {
                'F'
            } else if s_upper.starts_with("P") {
                'P'
            } else if s_upper.starts_with("E") {
                'E'
            } else if s_upper.starts_with("A") {
                'A'
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };
        
        match type_char {
            'F' => self.parse_fm_instrument(ast),
            'P' => self.parse_pcm_instrument(ast),
            'E' => self.parse_envelope_definition(ast),
            'A' => self.parse_arpeggio_definition(ast),
            _ => Ok(()),
        }
    }

    /// Parse FM instrument definition
    fn parse_fm_instrument(&mut self, ast: &mut MmlAst) -> MmlResult<()> {
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
        
        // Skip whitespace/commas
        while let Some(token) = self.current_token() {
            match token {
                Token::Comma | Token::Whitespace(_) => self.advance(),
                _ => break,
            }
        }
        
        let name = if let Some(Token::StringLiteral(s)) = self.current_token() {
            self.advance();
            s
        } else if let Some(Token::Identifier(s)) = self.current_token() {
            self.advance();
            s
        } else {
            String::new()
        };
        
        // Skip whitespace/commas
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
        
        let inst = FmInstrument { number, name, parameters };
        ast.fm_instruments.insert(number, inst);
        
        Ok(())
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

    /// Parse instrument selection
    fn parse_instrument_selection(&mut self) -> MmlResult<Option<MmlNode>> {
        if matches!(self.current_token(), Some(Token::AtSign)) {
            self.advance();
        }
        
        let number = if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            n as usize
        } else if let Some(Token::Identifier(s)) = self.current_token() {
            if s.parse::<usize>().is_ok() {
                self.advance();
                s.parse::<usize>().unwrap()
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None);
        };
        
        Ok(Some(MmlNode::InstrumentSelection(crate::compiler::ast::InstrumentSelection { number })))
    }

    /// Parse MML commands
    fn parse_mml_command(&mut self) -> MmlResult<Option<MmlNode>> {
        if let Some(token) = self.current_token() {
            match token {
                Token::Note(letter) => {
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
                    
                    Ok(Some(MmlNode::Note(note)))
                }
                
                Token::Rest => {
                    self.advance();
                    let mut rest = Rest {
                        duration: self.current_length,
                        dotted: false,
                    };
                    
                    if let Some(Token::Number(d)) = self.current_token() {
                        rest.duration = d;
                        self.advance();
                    }
                    
                    if self.consume(Token::Dot) {
                        rest.dotted = true;
                    }
                    
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
                    self.advance();
                    if let Some(Token::Number(n)) = self.current_token() {
                        let number = n as usize;
                        self.advance();
                        Ok(Some(MmlNode::InstrumentSelection(crate::compiler::ast::InstrumentSelection { number })))
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
                
                Token::Comment(text) => {
                    self.advance();
                    Ok(Some(MmlNode::Comment(text)))
                }
                
                _ => {
                    self.advance();
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
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
    let source = std::fs::read_to_string(path)?;
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
}
