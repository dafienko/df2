use superconsole::{Component, Dimensions, DrawMode, Lines};

pub struct LinesComponent {
    lines: Lines,
    fill_width: bool,
}

impl LinesComponent {
    pub fn new(lines: Lines, fill_width: bool) -> Self {
        Self { lines, fill_width }
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
