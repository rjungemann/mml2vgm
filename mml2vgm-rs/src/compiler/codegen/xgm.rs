//! XGM Format Generator
//!
//! This module generates XGM (Extended Game Music) format files from MML AST.
//!
//! # XGM Format Overview
//!
//! XGM is an extended version of VGM that supports the Sega Mega Drive/Genesis
//! sound chip configuration (YM2612 + SN76489). It uses a different command
//! structure optimized for this specific chip combination.

use super::{CodeGenerator, OutputFormat, VgmHeader};
use crate::compiler::ast::MmlAst;
use crate::{CompileOptions, MmlError, MmlResult, SoundChip};

/// XGM generator for standard Mega Drive configuration
pub struct XgmGenerator {
    header: VgmHeader,
    chips: Vec<SoundChip>,
    /// XGM-specific: PCM channel overlay data
    pcm_overlay: Vec<u8>,
}

/// XGM2 generator with extended PCM frequency support
pub struct Xgm2Generator {
    header: VgmHeader,
    chips: Vec<SoundChip>,
    /// XGM2-specific: PCM frequency data
    pcm_frequency: Vec<u8>,
}

impl XgmGenerator {
    /// Create a new XGM generator from an AST
    pub fn from_ast(_ast: &MmlAst, _options: &CompileOptions) -> MmlResult<Self> {
        let mut generator = Self {
            header: VgmHeader::default(),
            chips: Vec::new(),
            pcm_overlay: Vec::new(),
        };

        // XGM always targets YM2612 + SN76489
        generator.chips = vec![SoundChip::YM2612, SoundChip::SN76489];

        // Update header for XGM
        generator.header.ident = [b'X', b'G', b'M', b' '];
        generator.header.version = 0x00000100; // XGM version 1.00

        Ok(generator)
    }

    /// Generate XGM binary
    pub fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();

        // Write header
        self.write_header(&mut output)?;

        // Write command data
        // XGM uses a compact command format
        self.write_commands(&mut output)?;

        // Write PCM overlay if present
        if !self.pcm_overlay.is_empty() {
            output.extend_from_slice(&self.pcm_overlay);
        }

        Ok(output)
    }

    /// Write XGM header
    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        // Write ident
        output.extend_from_slice(&self.header.ident);

        // Write version
        output.extend_from_slice(&self.header.version.to_le_bytes());

        // XGM-specific header fields would go here
        // For now, write placeholder data to reach minimum header size
        while output.len() < 0x20 {
            output.push(0);
        }

        Ok(())
    }

    /// Write XGM commands
    fn write_commands(&self, _output: &mut Vec<u8>) -> MmlResult<()> {
        // XGM uses a compact command format:
        // - 1 byte: command and parameters
        // - Commands can write to YM2612 port 0, YM2612 port 1, or SN76489
        
        // Placeholder: Write a simple note command
        // This will be implemented fully in future work
        
        Ok(())
    }
}

impl Xgm2Generator {
    /// Create a new XGM2 generator from an AST
    pub fn from_ast(_ast: &MmlAst, _options: &CompileOptions) -> MmlResult<Self> {
        let mut generator = Self {
            header: VgmHeader::default(),
            chips: Vec::new(),
            pcm_frequency: Vec::new(),
        };

        // XGM2 always targets YM2612 + SN76489
        generator.chips = vec![SoundChip::YM2612, SoundChip::SN76489];

        // Update header for XGM2
        generator.header.ident = [b'X', b'G', b'M', b'2'];
        generator.header.version = 0x00000100; // XGM2 version 1.00

        Ok(generator)
    }

    /// Generate XGM2 binary
    pub fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();

        // Write header
        self.write_header(&mut output)?;

        // Write command data
        self.write_commands(&mut output)?;

        // Write PCM frequency data
        if !self.pcm_frequency.is_empty() {
            output.extend_from_slice(&self.pcm_frequency);
        }

        Ok(output)
    }

    /// Write XGM2 header
    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        // Write ident
        output.extend_from_slice(&self.header.ident);

        // Write version
        output.extend_from_slice(&self.header.version.to_le_bytes());

        // XGM2-specific header fields would go here
        while output.len() < 0x20 {
            output.push(0);
        }

        Ok(())
    }

    /// Write XGM2 commands
    fn write_commands(&self, _output: &mut Vec<u8>) -> MmlResult<()> {
        // XGM2 uses similar compact format to XGM
        // with additional PCM frequency support
        
        // Placeholder implementation
        
        Ok(())
    }
}

impl CodeGenerator for XgmGenerator {
    fn generate(&self) -> MmlResult<Vec<u8>> {
        self.generate()
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Xgm
    }

    fn chips(&self) -> &[SoundChip] {
        &self.chips
    }
}

impl CodeGenerator for Xgm2Generator {
    fn generate(&self) -> MmlResult<Vec<u8>> {
        self.generate()
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Xgm2
    }

    fn chips(&self) -> &[SoundChip] {
        &self.chips
    }
}
