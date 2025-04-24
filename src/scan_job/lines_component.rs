use superconsole::{Component, Dimensions, DrawMode, Lines};

pub struct LinesComponent {
    lines: Lines,
    fill_width: bool,
}

impl LinesComponent {
    pub fn new(lines: Lines) -> Self {
        Self {
            lines,
            fill_width: false,
        }
    }

    pub fn with_fill_width(mut self, fill_width: bool) -> Self {
        self.fill_width = fill_width;
        self
    }

    pub fn from_str(s: &str) -> Self {
        Self::new(Lines::from_colored_multiline_string(s))
    }
}

impl Component for LinesComponent {
    fn draw_unchecked(&self, dimensions: Dimensions, _mode: DrawMode) -> anyhow::Result<Lines> {
        let mut lines = self.lines.clone();
        if self.fill_width {
            lines.set_lines_to_exact_width(dimensions.width);
        }
        Ok(lines)
    }
}
