//! Diagnostic reporting with colored output

use super::{Error, Span};
use std::fmt;

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Diagnostic message with source location
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub source_name: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span: None,
            source_name: None,
        }
    }
    
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            span: None,
            source_name: None,
        }
    }
    
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    
    pub fn with_source(mut self, source_name: impl Into<String>) -> Self {
        self.source_name = Some(source_name.into());
        self
    }
    
    /// Format diagnostic with colored output
    pub fn format_with_source(&self, source: &str) -> String {
        let mut output = String::new();
        
        // Color codes
        let red = "\x1b[31m";
        let yellow = "\x1b[33m";
        let blue = "\x1b[34m";
        let bold = "\x1b[1m";
        let reset = "\x1b[0m";
        
        let (color, severity_text) = match self.severity {
            Severity::Error => (red, "error"),
            Severity::Warning => (yellow, "warning"),
            Severity::Info => (blue, "info"),
        };
        
        // Header line
        output.push_str(&format!("{}{}:{} {}{}\n", 
            bold, color, severity_text, self.message, reset));
        
        if let Some(span) = self.span {
            // Source name and location
            if let Some(ref source_name) = self.source_name {
                output.push_str(&format!("  --> {}:{}:{}\n", 
                    source_name, span.line, span.column));
            } else {
                output.push_str(&format!("  --> line {}:{}\n", 
                    span.line, span.column));
            }
            
            // Extract the relevant line from source
            if let Some(line_text) = get_line_from_source(source, span.line as usize) {
                let line_num_width = format!("{}", span.line).len();
                
                // Line number and source
                output.push_str(&format!("{:width$} | \n", "", width = line_num_width));
                output.push_str(&format!("{} | {}\n", span.line, line_text));
                
                // Error indicator
                let spaces = " ".repeat(line_num_width);
                let mut indicator = String::new();
                indicator.push_str(&format!("{} | ", spaces));
                
                // Add spaces to align with error column
                for _ in 0..span.column.saturating_sub(1) {
                    indicator.push(' ');
                }
                
                // Add error markers
                let error_len = if span.len() > 0 { span.len() } else { 1 };
                for _ in 0..error_len {
                    indicator.push_str(&format!("{}^{}", color, reset));
                }
                
                output.push_str(&indicator);
                output.push('\n');
            }
        }
        
        output
    }
}

impl From<Error> for Diagnostic {
    fn from(error: Error) -> Self {
        match error {
            Error::Lexer { message, span, .. } => {
                Diagnostic::error(message).with_span(span)
            },
            Error::Parser { message, span, .. } => {
                Diagnostic::error(message).with_span(span)
            },
            Error::Runtime { message, span } => {
                let mut diag = Diagnostic::error(message);
                if let Some(span) = span {
                    diag = diag.with_span(span);
                }
                diag
            },
            Error::Io(io_error) => {
                Diagnostic::error(format!("IO error: {}", io_error))
            },
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", 
            match self.severity {
                Severity::Error => "error",
                Severity::Warning => "warning", 
                Severity::Info => "info",
            },
            self.message
        )
    }
}

/// Extract a specific line from source text
fn get_line_from_source(source: &str, line_number: usize) -> Option<&str> {
    source
        .lines()
        .nth(line_number.saturating_sub(1))
}