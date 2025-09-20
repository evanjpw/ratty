use super::*;
use std::collections::HashMap;

/// Terminal emulation core
#[derive(Debug)]
pub struct Terminal {
    parser: VtParser,
    interpreter: VtInterpreter,
    current_mode: TerminalMode,
}

impl Terminal {
    /// Create a new terminal emulator
    pub fn new() -> PaneResult<Self> {
        Ok(Terminal {
            parser: VtParser::new(),
            interpreter: VtInterpreter::new(),
            current_mode: TerminalMode::Normal,
        })
    }
    
    /// Process a single byte of input
    pub fn process_byte(&mut self, byte: u8) -> PaneResult<Option<VtCommand>> {
        // First, feed the byte to the parser
        if let Some(sequence) = self.parser.process_byte(byte)? {
            // If we got a complete sequence, interpret it
            return self.interpreter.interpret(sequence);
        }
        
        // No complete sequence yet
        Ok(None)
    }
    
    /// Process multiple bytes of input
    pub fn process_bytes(&mut self, bytes: &[u8]) -> PaneResult<Vec<VtCommand>> {
        let mut commands = Vec::new();
        
        for &byte in bytes {
            if let Some(command) = self.process_byte(byte)? {
                commands.push(command);
            }
        }
        
        Ok(commands)
    }
    
    /// Get current terminal mode
    pub fn current_mode(&self) -> TerminalMode {
        self.current_mode
    }
    
    /// Set terminal mode
    pub fn set_mode(&mut self, mode: TerminalMode) {
        self.current_mode = mode;
    }
}

/// VT sequence parser
#[derive(Debug)]
pub struct VtParser {
    state_machine: ParserStateMachine,
    current_sequence: Vec<u8>,
    params: Vec<i32>,
}

impl VtParser {
    pub fn new() -> Self {
        VtParser {
            state_machine: ParserStateMachine::Ground,
            current_sequence: Vec::new(),
            params: Vec::new(),
        }
    }
    
    /// Process a byte and return a complete sequence if ready
    pub fn process_byte(&mut self, byte: u8) -> PaneResult<Option<VtSequence>> {
        match self.state_machine {
            ParserStateMachine::Ground => self.process_ground_state(byte),
            ParserStateMachine::Escape => self.process_escape_state(byte),
            ParserStateMachine::CsiEntry => self.process_csi_entry_state(byte),
            ParserStateMachine::CsiParam => self.process_csi_param_state(byte),
            ParserStateMachine::CsiIntermediate => self.process_csi_intermediate_state(byte),
            ParserStateMachine::OscString => self.process_osc_string_state(byte),
        }
    }
    
    fn process_ground_state(&mut self, byte: u8) -> PaneResult<Option<VtSequence>> {
        match byte {
            0x1B => {
                // ESC - start escape sequence
                self.state_machine = ParserStateMachine::Escape;
                self.current_sequence.clear();
                self.current_sequence.push(byte);
                Ok(None)
            }
            0x08 => Ok(Some(VtSequence::Control(ControlCode::Backspace))),
            0x09 => Ok(Some(VtSequence::Control(ControlCode::Tab))),
            0x0A => Ok(Some(VtSequence::Control(ControlCode::LineFeed))),
            0x0D => Ok(Some(VtSequence::Control(ControlCode::CarriageReturn))),
            0x07 => Ok(Some(VtSequence::Control(ControlCode::Bell))),
            0x20..=0x7E => {
                // Printable ASCII
                Ok(Some(VtSequence::Character(byte as char)))
            }
            0x80..=0xFF => {
                // Extended characters - for now, treat as printable
                // TODO: Proper UTF-8 handling
                Ok(Some(VtSequence::Character(byte as char)))
            }
            _ => {
                // Other control characters - ignore for now
                Ok(None)
            }
        }
    }
    
    fn process_escape_state(&mut self, byte: u8) -> PaneResult<Option<VtSequence>> {
        self.current_sequence.push(byte);
        
        match byte {
            b'[' => {
                // CSI sequence
                self.state_machine = ParserStateMachine::CsiEntry;
                Ok(None)
            }
            b']' => {
                // OSC sequence
                self.state_machine = ParserStateMachine::OscString;
                Ok(None)
            }
            b'D' => {
                // Index (move down)
                self.state_machine = ParserStateMachine::Ground;
                Ok(Some(VtSequence::Escape(EscapeSequence::Index)))
            }
            b'M' => {
                // Reverse Index (move up)
                self.state_machine = ParserStateMachine::Ground;
                Ok(Some(VtSequence::Escape(EscapeSequence::ReverseIndex)))
            }
            b'c' => {
                // Reset
                self.state_machine = ParserStateMachine::Ground;
                Ok(Some(VtSequence::Escape(EscapeSequence::Reset)))
            }
            b'7' => {
                // Save cursor
                self.state_machine = ParserStateMachine::Ground;
                Ok(Some(VtSequence::Escape(EscapeSequence::SaveCursor)))
            }
            b'8' => {
                // Restore cursor
                self.state_machine = ParserStateMachine::Ground;
                Ok(Some(VtSequence::Escape(EscapeSequence::RestoreCursor)))
            }
            _ => {
                // Unknown escape sequence - return to ground state
                self.state_machine = ParserStateMachine::Ground;
                Ok(None)
            }
        }
    }
    
    fn process_csi_entry_state(&mut self, byte: u8) -> PaneResult<Option<VtSequence>> {
        self.current_sequence.push(byte);
        
        match byte {
            b'0'..=b'9' | b';' => {
                // Parameter bytes
                self.state_machine = ParserStateMachine::CsiParam;
                self.parse_parameters()
            }
            b'A'..=b'Z' | b'a'..=b'z' => {
                // Final byte
                let sequence = self.build_csi_sequence(byte)?;
                self.state_machine = ParserStateMachine::Ground;
                Ok(Some(sequence))
            }
            _ => {
                // Invalid - return to ground
                self.state_machine = ParserStateMachine::Ground;
                Ok(None)
            }
        }
    }
    
    fn process_csi_param_state(&mut self, byte: u8) -> PaneResult<Option<VtSequence>> {
        self.current_sequence.push(byte);
        
        match byte {
            b'0'..=b'9' | b';' => {
                // More parameter bytes
                self.parse_parameters()
            }
            b'A'..=b'Z' | b'a'..=b'z' => {
                // Final byte
                let sequence = self.build_csi_sequence(byte)?;
                self.state_machine = ParserStateMachine::Ground;
                Ok(Some(sequence))
            }
            _ => {
                // Invalid - return to ground
                self.state_machine = ParserStateMachine::Ground;
                Ok(None)
            }
        }
    }
    
    fn process_csi_intermediate_state(&mut self, byte: u8) -> PaneResult<Option<VtSequence>> {
        // TODO: Handle intermediate bytes
        self.current_sequence.push(byte);
        self.state_machine = ParserStateMachine::Ground;
        Ok(None)
    }
    
    fn process_osc_string_state(&mut self, byte: u8) -> PaneResult<Option<VtSequence>> {
        self.current_sequence.push(byte);
        
        // OSC sequences end with ST (String Terminator) or BEL
        if byte == 0x07 || (self.current_sequence.len() >= 2 && 
                            self.current_sequence[self.current_sequence.len()-2] == 0x1B && 
                            byte == b'\\') {
            // End of OSC sequence
            self.state_machine = ParserStateMachine::Ground;
            // TODO: Parse OSC sequence
            Ok(None)
        } else {
            Ok(None)
        }
    }
    
    fn parse_parameters(&mut self) -> PaneResult<Option<VtSequence>> {
        // Extract parameter string (everything after CSI '[' until now)
        if self.current_sequence.len() < 3 {
            return Ok(None);
        }
        
        let param_start = 2; // After ESC and '['
        let param_end = self.current_sequence.len() - 1; // Before final byte
        let param_str = std::str::from_utf8(&self.current_sequence[param_start..param_end])
            .map_err(|_| PaneError::ParseError("Invalid UTF-8 in CSI parameters".to_string()))?;
        
        self.params.clear();
        if !param_str.is_empty() {
            for param in param_str.split(';') {
                if param.is_empty() {
                    self.params.push(0); // Default parameter
                } else {
                    self.params.push(param.parse().unwrap_or(0));
                }
            }
        }
        
        Ok(None)
    }
    
    fn build_csi_sequence(&self, final_byte: u8) -> PaneResult<VtSequence> {
        let params = self.params.clone();
        
        let csi_command = match final_byte {
            b'A' => CsiCommand::CursorUp(params.first().copied().unwrap_or(1) as u16),
            b'B' => CsiCommand::CursorDown(params.first().copied().unwrap_or(1) as u16),
            b'C' => CsiCommand::CursorForward(params.first().copied().unwrap_or(1) as u16),
            b'D' => CsiCommand::CursorBack(params.first().copied().unwrap_or(1) as u16),
            b'H' | b'f' => {
                let row = params.get(0).copied().unwrap_or(1) as u16;
                let col = params.get(1).copied().unwrap_or(1) as u16;
                CsiCommand::CursorPosition(row, col)
            }
            b'J' => {
                let param = params.first().copied().unwrap_or(0);
                let clear_type = match param {
                    0 => ClearType::ToEnd,
                    1 => ClearType::ToBeginning,
                    2 => ClearType::All,
                    _ => ClearType::All,
                };
                CsiCommand::ClearScreen(clear_type)
            }
            b'K' => {
                let param = params.first().copied().unwrap_or(0);
                let clear_type = match param {
                    0 => ClearType::ToEnd,
                    1 => ClearType::ToBeginning,
                    2 => ClearType::All,
                    _ => ClearType::All,
                };
                CsiCommand::ClearLine(clear_type)
            }
            b'm' => CsiCommand::SetGraphicsRendition(params.iter().map(|&p| p as u8).collect()),
            b'L' => CsiCommand::InsertLines(params.first().copied().unwrap_or(1) as u16),
            b'M' => CsiCommand::DeleteLines(params.first().copied().unwrap_or(1) as u16),
            _ => CsiCommand::Unknown(final_byte, params),
        };
        
        Ok(VtSequence::Csi(csi_command))
    }
}

/// Parser state machine states
#[derive(Debug, Clone, Copy)]
enum ParserStateMachine {
    Ground,
    Escape,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    OscString,
}

/// Complete VT sequence types
#[derive(Debug, Clone)]
pub enum VtSequence {
    Character(char),
    Control(ControlCode),
    Escape(EscapeSequence),
    Csi(CsiCommand),
    Osc(OscCommand),
}

#[derive(Debug, Clone)]
pub enum ControlCode {
    Bell,
    Backspace,
    Tab,
    LineFeed,
    CarriageReturn,
}

#[derive(Debug, Clone)]
pub enum EscapeSequence {
    Index,
    ReverseIndex,
    Reset,
    SaveCursor,
    RestoreCursor,
}

#[derive(Debug, Clone)]
pub enum CsiCommand {
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBack(u16),
    CursorPosition(u16, u16),
    ClearScreen(ClearType),
    ClearLine(ClearType),
    InsertLines(u16),
    DeleteLines(u16),
    SetGraphicsRendition(Vec<u8>),
    Unknown(u8, Vec<i32>),
}

#[derive(Debug, Clone)]
pub enum OscCommand {
    SetTitle(String),
    Unknown(Vec<u8>),
}

/// VT command interpreter
pub struct VtInterpreter {
    handler_map: HashMap<String, Box<dyn VtCommandHandler>>,
}

impl std::fmt::Debug for VtInterpreter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VtInterpreter")
            .field("handler_count", &self.handler_map.len())
            .finish()
    }
}

impl VtInterpreter {
    pub fn new() -> Self {
        VtInterpreter {
            handler_map: HashMap::new(),
        }
    }
    
    /// Interpret a VT sequence into a command
    pub fn interpret(&self, sequence: VtSequence) -> PaneResult<Option<VtCommand>> {
        match sequence {
            VtSequence::Character(ch) => Ok(Some(VtCommand::PrintChar(ch))),
            VtSequence::Control(code) => self.interpret_control(code),
            VtSequence::Escape(esc) => self.interpret_escape(esc),
            VtSequence::Csi(csi) => self.interpret_csi(csi),
            VtSequence::Osc(osc) => self.interpret_osc(osc),
        }
    }
    
    fn interpret_control(&self, code: ControlCode) -> PaneResult<Option<VtCommand>> {
        let command = match code {
            ControlCode::Bell => VtCommand::Bell,
            ControlCode::Backspace => VtCommand::Backspace,
            ControlCode::Tab => VtCommand::Tab,
            ControlCode::LineFeed => VtCommand::LineFeed,
            ControlCode::CarriageReturn => VtCommand::CarriageReturn,
        };
        Ok(Some(command))
    }
    
    fn interpret_escape(&self, esc: EscapeSequence) -> PaneResult<Option<VtCommand>> {
        let command = match esc {
            EscapeSequence::Index => VtCommand::CursorDown(1),
            EscapeSequence::ReverseIndex => VtCommand::CursorUp(1),
            EscapeSequence::Reset => VtCommand::Reset,
            EscapeSequence::SaveCursor => VtCommand::SaveCursor,
            EscapeSequence::RestoreCursor => VtCommand::RestoreCursor,
        };
        Ok(Some(command))
    }
    
    fn interpret_csi(&self, csi: CsiCommand) -> PaneResult<Option<VtCommand>> {
        let command = match csi {
            CsiCommand::CursorUp(n) => VtCommand::CursorUp(n),
            CsiCommand::CursorDown(n) => VtCommand::CursorDown(n),
            CsiCommand::CursorForward(n) => VtCommand::CursorForward(n),
            CsiCommand::CursorBack(n) => VtCommand::CursorBack(n),
            CsiCommand::CursorPosition(row, col) => VtCommand::CursorPosition(row, col),
            CsiCommand::ClearScreen(clear_type) => VtCommand::ClearScreen(clear_type),
            CsiCommand::ClearLine(clear_type) => VtCommand::ClearLine(clear_type),
            CsiCommand::InsertLines(n) => VtCommand::InsertLines(n),
            CsiCommand::DeleteLines(n) => VtCommand::DeleteLines(n),
            CsiCommand::SetGraphicsRendition(params) => VtCommand::SetGraphicsRendition(params),
            CsiCommand::Unknown(_, _) => return Ok(None), // Skip unknown commands
        };
        Ok(Some(command))
    }
    
    fn interpret_osc(&self, _osc: OscCommand) -> PaneResult<Option<VtCommand>> {
        // TODO: Implement OSC command interpretation
        Ok(None)
    }
}

/// High-level terminal commands
#[derive(Debug, Clone)]
pub enum VtCommand {
    // Character handling
    PrintChar(char),
    Bell,
    Backspace,
    Tab,
    LineFeed,
    CarriageReturn,
    
    // Cursor movement
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBack(u16),
    CursorPosition(u16, u16),
    SaveCursor,
    RestoreCursor,
    
    // Screen manipulation
    ClearScreen(ClearType),
    ClearLine(ClearType),
    InsertLines(u16),
    DeleteLines(u16),
    
    // Attributes
    SetGraphicsRendition(Vec<u8>),
    
    // Terminal control
    Reset,
    
    // Mode changes
    SetMode(Vec<u16>),
    ResetMode(Vec<u16>),
    DecPrivateModeSet(u16),
    DecPrivateModeReset(u16),
    
    // Advanced features
    DeviceStatusReport,
}

/// Terminal operating mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalMode {
    Normal,
    ApplicationKeypad,
    ApplicationCursor,
}

/// VT command handler trait
pub trait VtCommandHandler: Send + Sync {
    fn handle(&self, command: VtCommand) -> PaneResult<()>;
    fn can_handle(&self, command: &VtCommand) -> bool;
}