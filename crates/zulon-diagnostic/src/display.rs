// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Display implementation for diagnostics

use crate::diagnostic::Diagnostic;
use crate::severity::Severity;
use crate::span::Span;
use std::collections::HashMap;

impl Diagnostic {
    /// Display this diagnostic with full source code context
    pub fn display_with_context(&self, source: &str, use_colors: bool) -> String {
        let mut output = String::new();

        // Print severity and code
        self.print_header(&mut output, use_colors);

        // Print location arrow
        if let Some(span) = &self.span {
            self.print_location_arrow(&mut output, span, use_colors);
        }

        // Print source code snippet with context
        if let Some(span) = &self.span {
            if !span.is_dummy() {
                self.print_source_snippet_with_context(&mut output, source, span, use_colors);
            }
        }

        // Print labels for additional spans
        let primary_span = self.span.clone();
        for label in &self.labels {
            if primary_span.as_ref().map_or(false, |ps| label.span != *ps) {
                self.print_labeled_span(&mut output, &label.span, &label.message, source, use_colors);
            }
        }

        // Print notes
        for note in &self.notes {
            self.print_note(&mut output, note, use_colors);
        }

        // Print suggestions
        for suggestion in &self.suggestions {
            self.print_suggestion_with_code(&mut output, suggestion, source, use_colors);
        }

        // Print related diagnostics
        for related in &self.related {
            output.push_str("\n");
            output.push_str(&related.display_with_context(source, use_colors));
        }

        output
    }

    fn print_header(&self, output: &mut String, use_colors: bool) {
        let severity_str = if use_colors {
            format!(
                "{}{}{}",
                self.severity.color_code(),
                self.severity.name(),
                Severity::reset_code()
            )
        } else {
            self.severity.name().to_string()
        };

        if let Some(code) = &self.code {
            output.push_str(&format!("{}[{}]: {}\n", severity_str, code, self.message));
        } else {
            output.push_str(&format!("{}: {}\n", severity_str, self.message));
        }
    }

    fn print_location_arrow(&self, output: &mut String, span: &Span, _use_colors: bool) {
        let location = format!("  --> {}", span);
        output.push_str(&location);
        output.push_str("\n");
    }

    fn print_source_snippet_with_context(&self, output: &mut String, source: &str, span: &Span, use_colors: bool) {
        let lines: Vec<&str> = source.lines().collect();

        // Determine context window (show 1 line before and after)
        let context_lines = 1;
        let start_line = span.lo.line.saturating_sub(context_lines);
        let end_line = (span.hi.line + context_lines).min(lines.len());

        // Group spans by line for multi-span marking
        let mut line_spans: HashMap<usize, Vec<(Span, Option<&str>)>> = HashMap::new();
        line_spans.insert(span.lo.line, vec![(span.clone(), Some("primary"))]);

        for label in &self.labels {
            line_spans.entry(label.span.lo.line)
                .or_insert_with(Vec::new)
                .push((label.span.clone(), Some(label.message.as_str())));
        }

        // Print source lines with context
        for line_num in start_line..=end_line {
            output.push_str(&format!("{:>3} | ", line_num));

            if line_num > 0 && line_num <= lines.len() {
                let line = lines[line_num - 1];
                output.push_str(line);
            }
            output.push_str("\n");

            // Print markers for this line
            if let Some(spans) = line_spans.get(&line_num) {
                output.push_str("   | ");

                // Calculate marker positions
                let mut marker_line = vec![' '; lines[line_num - 1].len()];

                for (mark_span, _label) in spans {
                    if mark_span.lo.line == line_num {
                        let col_start = mark_span.lo.column.saturating_sub(1);
                        let col_end = mark_span.hi.column.saturating_sub(1).min(marker_line.len());
                        let width = if col_end >= col_start { col_end - col_start + 1 } else { 1 };

                        let marker_char = if mark_span.lo.line == span.lo.line && mark_span.lo.offset == span.lo.offset { '^' } else { '-' };
                        let _width = width;  // suppress unused warning

                        for i in col_start..col_end.min(marker_line.len()) {
                            marker_line[i] = marker_char;
                        }
                    }
                }

                let marker: String = marker_line.iter().collect();
                let colored_marker = if use_colors {
                    format!("{}{}{}", "\x1b[31m", marker, "\x1b[0m")
                } else {
                    marker
                };

                output.push_str(&colored_marker);
                output.push_str("\n");

                // Print labels below markers
                for (mark_span, label) in spans {
                    if mark_span.lo.line == line_num && mark_span.lo.offset == span.lo.offset {
                        if let Some(msg) = label {
                            output.push_str("   |      ");
                            output.push_str(msg);
                            output.push_str("\n");
                        }
                    }
                }
            }
        }
    }

    fn print_labeled_span(&self, output: &mut String, span: &Span, label: &str, source: &str, use_colors: bool) {
        if span.is_dummy() {
            return;
        }

        let lines: Vec<&str> = source.lines().collect();
        let line_num = span.lo.line;

        if line_num == 0 || line_num > lines.len() {
            return;
        }

        output.push_str(&format!("{:>3} | ", line_num));
        output.push_str(lines[line_num - 1]);
        output.push_str("\n");

        output.push_str("   | ");

        let col_start = span.lo.column.saturating_sub(1);
        let col_end = span.hi.column.saturating_sub(1);
        let width = if col_end >= col_start { col_end - col_start + 1 } else { 1 };

        for _ in 0..col_start {
            output.push(' ');
        }

        let marker = if use_colors {
            format!("{}{}{}", "\x1b[33m", "-".repeat(width), "\x1b[0m")
        } else {
            "-".repeat(width)
        };

        output.push_str(&marker);
        output.push_str(&format!(" {}", label));
        output.push_str("\n");
    }

    fn print_note(&self, output: &mut String, note: &str, use_colors: bool) {
        let note_str = if use_colors {
            format!("{}note{}: {}", "\x1b[36m", "\x1b[0m", note)
        } else {
            format!("note: {}", note)
        };
        output.push_str(&note_str);
        output.push_str("\n");
    }

    fn print_suggestion_with_code(&self, output: &mut String, suggestion: &crate::suggestion::Suggestion, source: &str, use_colors: bool) {
        let help_str = if use_colors {
            format!("{}help{}: {}", "\x1b[32m", "\x1b[0m", suggestion.message)
        } else {
            format!("help: {}", suggestion.message)
        };
        output.push_str(&help_str);
        output.push_str("\n");

        // Show the suggested replacement if span is valid
        if !suggestion.span.is_dummy() {
            let lines: Vec<&str> = source.lines().collect();
            let line_num = suggestion.span.lo.line;

            if line_num > 0 && line_num <= lines.len() {
                output.push_str(&format!("{:>3} | ", line_num));
                output.push_str("    ");  // indentation for suggestion

                // Show the line with replacement
                let line = lines[line_num - 1];
                let start = suggestion.span.lo.offset;
                let end = suggestion.span.hi.offset;

                if start <= line.len() && end <= line.len() && start <= end {
                    output.push_str(&line[..start]);
                    if use_colors {
                        output.push_str(&format!("{}{}{}", "\x1b[32m", suggestion.replacement, "\x1b[0m"));
                    } else {
                        output.push_str(&suggestion.replacement);
                    }
                    if end < line.len() {
                        output.push_str(&line[end..]);
                    }
                } else {
                    output.push_str(line);
                }
                output.push_str("\n");
            }
        }
    }
}
