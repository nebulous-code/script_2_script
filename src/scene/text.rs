use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleFlags {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl StyleFlags {
    pub const PLAIN: StyleFlags = StyleFlags {
        bold: false,
        italic: false,
        underline: false,
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub style: StyleFlags,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyledText {
    pub runs: Vec<TextRun>,
}

impl StyledText {
    pub fn from_markdown(input: &str) -> Self {
        let mut runs = Vec::new();
        let mut buffer = String::new();
        let mut style = StyleFlags::PLAIN;

        let mut i = 0;
        let chars: Vec<char> = input.chars().collect();
        while i < chars.len() {
            let next_two = if i + 1 < chars.len() {
                Some((chars[i], chars[i + 1]))
            } else {
                None
            };

            let mut matched = false;
            if let Some((a, b)) = next_two {
                if a == '*' && b == '*' {
                    flush_run(&mut runs, &mut buffer, style);
                    style.bold = !style.bold;
                    i += 2;
                    matched = true;
                } else if a == '_' && b == '_' {
                    flush_run(&mut runs, &mut buffer, style);
                    style.underline = !style.underline;
                    i += 2;
                    matched = true;
                }
            }

            if matched {
                continue;
            }

            if chars[i] == '*' {
                flush_run(&mut runs, &mut buffer, style);
                style.italic = !style.italic;
                i += 1;
                continue;
            }

            buffer.push(chars[i]);
            i += 1;
        }

        flush_run(&mut runs, &mut buffer, style);
        Self { runs }
    }
}

fn flush_run(runs: &mut Vec<TextRun>, buffer: &mut String, style: StyleFlags) {
    if !buffer.is_empty() {
        runs.push(TextRun {
            text: buffer.clone(),
            style,
        });
        buffer.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontSource {
    Default,
    Path(PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontFamily {
    pub regular: FontSource,
    pub bold: Option<FontSource>,
    pub italic: Option<FontSource>,
    pub bold_italic: Option<FontSource>,
}

impl FontFamily {
    pub fn default() -> Self {
        Self {
            regular: FontSource::Default,
            bold: None,
            italic: None,
            bold_italic: None,
        }
    }

    pub fn resolve(&self, style: StyleFlags) -> &FontSource {
        if style.bold && style.italic {
            if let Some(font) = &self.bold_italic {
                return font;
            }
        }
        if style.bold {
            if let Some(font) = &self.bold {
                return font;
            }
        }
        if style.italic {
            if let Some(font) = &self.italic {
                return font;
            }
        }
        &self.regular
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextObject {
    pub text: StyledText,
    pub font: FontFamily,
    pub font_size: f32,
    pub spacing: f32,
    pub max_width: f32,
    pub color: crate::scene::Color,
    pub line_spacing: f32,
}
