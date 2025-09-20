use super::*;
use crate::pane::{Cell, Cursor, CursorStyle, CursorVisibility, Line};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color as RatatuiColor, Modifier, Style as RatatuiStyle},
    text::{Line as RatatuiLine, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame as RatatuiFrame,
};
use std::time::Instant;

/// Terminal content renderer using ratatui
#[derive(Debug)]
pub struct TerminalRenderer {
    text_renderer: TextRenderer,
    cursor_renderer: CursorRenderer,
    decoration_renderer: DecorationRenderer,
    pub(crate) config: RendererConfig,
}

impl TerminalRenderer {
    /// Create a new terminal renderer
    pub fn new(config: &GlazingConfig) -> GlazingResult<Self> {
        Ok(TerminalRenderer {
            text_renderer: TextRenderer::new(&config.renderer)?,
            cursor_renderer: CursorRenderer::new(&config.cursor)?,
            decoration_renderer: DecorationRenderer::new()?,
            config: config.renderer.clone(),
        })
    }
    
    /// Render a complete frame
    pub fn render_frame<B: Backend>(
        &mut self,
        frame: &mut RatatuiFrame,
        area: Rect,
        render_frame: &RenderFrame,
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<()> {
        // Clear the area if needed
        if render_frame.needs_redraw() {
            frame.render_widget(Clear, area);
        }
        
        // Render the main content
        let content_area = self.decoration_renderer.render_border::<B>(frame, area, theme_engine)?;
        self.render_content::<B>(frame, content_area, render_frame, theme_engine)?;
        
        // Render cursor if present
        if let Some(ref cursor) = render_frame.cursor {
            self.cursor_renderer.render::<B>(frame, content_area, cursor, theme_engine)?;
        }
        
        Ok(())
    }
    
    /// Render a single line
    pub fn render_line(
        &self,
        line: &Line,
        line_number: usize,
        theme_engine: &ThemeEngine,
        has_cursor: bool,
    ) -> GlazingResult<RenderedLine> {
        let rendered_cells = self.text_renderer.render_cells(&line.cells, theme_engine)?;
        
        Ok(RenderedLine {
            cells: rendered_cells,
            line_number,
            is_wrapped: line.wrapped,
            dirty: line.dirty,
            has_cursor,
            text_spans: self.text_renderer.create_spans(&line.cells, theme_engine)?,
        })
    }
    
    /// Render cursor
    pub fn render_cursor(
        &self,
        cursor: &Cursor,
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<RenderedCursor> {
        // Create a temporary cursor renderer since we need mutable access
        let mut temp_renderer = self.cursor_renderer.clone();
        temp_renderer.create_rendered_cursor(cursor, theme_engine)
    }
    
    /// Update renderer configuration
    pub fn update_config(&mut self, config: &GlazingConfig) -> GlazingResult<()> {
        self.config = config.renderer.clone();
        self.text_renderer.update_config(&config.renderer)?;
        self.cursor_renderer.update_config(&config.cursor)?;
        Ok(())
    }
    
    /// Render the main terminal content
    fn render_content<B: Backend>(
        &self,
        frame: &mut RatatuiFrame,
        area: Rect,
        render_frame: &RenderFrame,
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<()> {
        // Create text widget from rendered lines
        let text_lines: Vec<RatatuiLine> = render_frame
            .content
            .iter()
            .map(|line| RatatuiLine::from(line.text_spans.clone()))
            .collect();
        
        let text = Text::from(text_lines);
        let paragraph = Paragraph::new(text)
            .style(theme_engine.get_base_style())
            .scroll((render_frame.viewport.scroll_offset as u16, 0));
        
        frame.render_widget(paragraph, area);
        
        Ok(())
    }
}

/// Text rendering component
#[derive(Debug)]
pub struct TextRenderer {
    config: RendererConfig,
}

impl TextRenderer {
    pub fn new(config: &RendererConfig) -> GlazingResult<Self> {
        Ok(TextRenderer {
            config: config.clone(),
        })
    }
    
    /// Render terminal cells to display cells
    pub fn render_cells(
        &self,
        cells: &[Cell],
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<Vec<RenderedCell>> {
        let mut rendered_cells = Vec::with_capacity(cells.len());
        
        for (col, cell) in cells.iter().enumerate() {
            let style = theme_engine.convert_cell_style(&cell.attributes, cell.foreground, cell.background)?;
            
            rendered_cells.push(RenderedCell {
                character: cell.character,
                style: style.clone(),
                position: CellPosition { col },
                ratatui_style: self.create_ratatui_style(&style)?,
            });
        }
        
        Ok(rendered_cells)
    }
    
    /// Create text spans for ratatui rendering
    pub fn create_spans(
        &self,
        cells: &[Cell],
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<Vec<Span<'static>>> {
        let mut spans = Vec::new();
        let mut current_text = String::new();
        let mut current_style = None;
        
        for cell in cells {
            let cell_style = theme_engine.convert_cell_style(
                &cell.attributes,
                cell.foreground,
                cell.background,
            )?;
            let ratatui_style = self.create_ratatui_style(&cell_style)?;
            
            // If style changed, flush current span and start new one
            if current_style.as_ref() != Some(&ratatui_style) {
                if !current_text.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current_text),
                        current_style.unwrap_or_default(),
                    ));
                }
                current_style = Some(ratatui_style);
            }
            
            current_text.push(cell.character);
        }
        
        // Flush remaining text
        if !current_text.is_empty() {
            spans.push(Span::styled(
                current_text,
                current_style.unwrap_or_default(),
            ));
        }
        
        Ok(spans)
    }
    
    /// Convert cell style to ratatui style
    fn create_ratatui_style(&self, style: &CellStyle) -> GlazingResult<RatatuiStyle> {
        let mut ratatui_style = RatatuiStyle::default()
            .fg(self.convert_color(style.foreground)?)
            .bg(self.convert_color(style.background)?);
        
        let mut modifiers = Modifier::empty();
        
        if style.bold {
            modifiers |= Modifier::BOLD;
        }
        if style.italic {
            modifiers |= Modifier::ITALIC;
        }
        if style.underline {
            modifiers |= Modifier::UNDERLINED;
        }
        if style.strikethrough {
            modifiers |= Modifier::CROSSED_OUT;
        }
        if style.reverse {
            modifiers |= Modifier::REVERSED;
        }
        if style.dim {
            modifiers |= Modifier::DIM;
        }
        
        ratatui_style = ratatui_style.add_modifier(modifiers);
        
        Ok(ratatui_style)
    }
    
    /// Convert color to ratatui color
    fn convert_color(&self, color: crate::sash::Color) -> GlazingResult<RatatuiColor> {
        Ok(RatatuiColor::Rgb(color.r, color.g, color.b))
    }
    
    /// Update text renderer configuration
    pub fn update_config(&mut self, config: &RendererConfig) -> GlazingResult<()> {
        self.config = config.clone();
        Ok(())
    }
}

/// Cursor rendering component
#[derive(Debug, Clone)]
pub struct CursorRenderer {
    config: CursorConfig,
    blink_state: BlinkState,
}

impl CursorRenderer {
    pub fn new(config: &CursorConfig) -> GlazingResult<Self> {
        Ok(CursorRenderer {
            config: config.clone(),
            blink_state: BlinkState::new(config.blink_rate),
        })
    }
    
    /// Render cursor to the frame
    pub fn render<B: Backend>(
        &mut self,
        frame: &mut RatatuiFrame,
        area: Rect,
        cursor: &RenderedCursor,
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<()> {
        if !cursor.visible || !self.should_show_cursor() {
            return Ok(());
        }
        
        let cursor_area = self.calculate_cursor_area(area, cursor)?;
        let cursor_widget = self.create_cursor_widget(cursor, theme_engine)?;
        
        frame.render_widget(cursor_widget, cursor_area);
        
        Ok(())
    }
    
    /// Create a rendered cursor from cursor state
    pub fn create_rendered_cursor(
        &mut self,
        cursor: &Cursor,
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<RenderedCursor> {
        let visible = match cursor.visibility {
            CursorVisibility::Visible => true,
            CursorVisibility::Hidden => false,
            CursorVisibility::BlinkingBlock |
            CursorVisibility::BlinkingUnderline |
            CursorVisibility::BlinkingBar => self.should_show_cursor(),
        };
        
        Ok(RenderedCursor {
            position: cursor.position,
            style: cursor.style,
            visible,
            color: theme_engine.get_cursor_color(),
            blink_phase: self.blink_state.current_phase(),
        })
    }
    
    /// Update cursor renderer configuration
    pub fn update_config(&mut self, config: &CursorConfig) -> GlazingResult<()> {
        self.config = config.clone();
        self.blink_state.update_rate(config.blink_rate);
        Ok(())
    }
    
    /// Check if cursor should be visible based on blink state
    fn should_show_cursor(&mut self) -> bool {
        if self.config.enable_blinking {
            self.blink_state.update_visibility()
        } else {
            true
        }
    }
    
    /// Calculate the area where cursor should be rendered
    fn calculate_cursor_area(&self, area: Rect, cursor: &RenderedCursor) -> GlazingResult<Rect> {
        let col = cursor.position.col.min(area.width.saturating_sub(1));
        let row = cursor.position.row.min(area.height.saturating_sub(1));
        
        let (width, height) = match cursor.style {
            CursorStyle::Block => (1, 1),
            CursorStyle::Underline => (1, 1), // Will be rendered as bottom part of cell
            CursorStyle::Bar => (1, 1), // Will be rendered as left part of cell
        };
        
        Ok(Rect {
            x: area.x + col,
            y: area.y + row,
            width,
            height,
        })
    }
    
    /// Create cursor widget for rendering
    fn create_cursor_widget(
        &self,
        cursor: &RenderedCursor,
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<impl ratatui::widgets::Widget> {
        let cursor_char = match cursor.style {
            CursorStyle::Block => '█',
            CursorStyle::Underline => '_',
            CursorStyle::Bar => '│',
        };
        
        let style = RatatuiStyle::default()
            .fg(cursor.color.into())
            .bg(theme_engine.get_background_color());
        
        Ok(Paragraph::new(cursor_char.to_string()).style(style))
    }
}

/// Decoration rendering component (borders, scrollbars, etc.)
#[derive(Debug)]
pub struct DecorationRenderer;

impl DecorationRenderer {
    pub fn new() -> GlazingResult<Self> {
        Ok(DecorationRenderer)
    }
    
    /// Render border around content area and return inner area
    pub fn render_border<B: Backend>(
        &self,
        frame: &mut RatatuiFrame,
        area: Rect,
        theme_engine: &ThemeEngine,
    ) -> GlazingResult<Rect> {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(theme_engine.get_border_style());
        
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        
        Ok(inner_area)
    }
}

/// Rendered line ready for display
#[derive(Debug, Clone)]
pub struct RenderedLine {
    pub cells: Vec<RenderedCell>,
    pub line_number: usize,
    pub is_wrapped: bool,
    pub dirty: bool,
    pub has_cursor: bool,
    pub text_spans: Vec<Span<'static>>,
}

/// Rendered cell with display information
#[derive(Debug, Clone)]
pub struct RenderedCell {
    pub character: char,
    pub style: CellStyle,
    pub position: CellPosition,
    pub ratatui_style: RatatuiStyle,
}

/// Cell position in the rendered output
#[derive(Debug, Clone, Copy)]
pub struct CellPosition {
    pub col: usize,
}

/// Rendered cursor ready for display
#[derive(Debug, Clone)]
pub struct RenderedCursor {
    pub position: crate::pane::CursorPosition,
    pub style: CursorStyle,
    pub visible: bool,
    pub color: crate::sash::Color,
    pub blink_phase: f32,
}

/// Cell styling information
#[derive(Debug, Clone, PartialEq)]
pub struct CellStyle {
    pub foreground: crate::sash::Color,
    pub background: crate::sash::Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub dim: bool,
}

/// Cursor blink state tracking
#[derive(Debug, Clone)]
pub struct BlinkState {
    rate: Duration,
    last_toggle: Instant,
    visible: bool,
}

impl BlinkState {
    pub fn new(rate: Duration) -> Self {
        BlinkState {
            rate,
            last_toggle: Instant::now(),
            visible: true,
        }
    }
    
    pub fn update_visibility(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_toggle) >= self.rate {
            self.visible = !self.visible;
            self.last_toggle = now;
        }
        self.visible
    }
    
    pub fn current_phase(&self) -> f32 {
        let elapsed = self.last_toggle.elapsed();
        (elapsed.as_millis() as f32 / self.rate.as_millis() as f32) % 1.0
    }
    
    pub fn update_rate(&mut self, new_rate: Duration) {
        self.rate = new_rate;
    }
}

/// Dirty region for efficient redraws
#[derive(Debug, Clone)]
pub enum DirtyRegion {
    Full,
    Lines { start: usize, end: usize },
    Cells { line: usize, start_col: usize, end_col: usize },
}