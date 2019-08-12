use rgb24::Rgb24;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Style {
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub foreground: Option<Rgb24>,
    pub background: Option<Rgb24>,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            bold: None,
            underline: None,
            foreground: Some(Rgb24::new_grey(255)),
            background: None,
        }
    }
    pub const fn with_bold(self, bold: bool) -> Self {
        Self {
            bold: Some(bold),
            ..self
        }
    }
    pub const fn with_underline(self, underline: bool) -> Self {
        Self {
            underline: Some(underline),
            ..self
        }
    }
    pub const fn with_foreground(self, foreground: Rgb24) -> Self {
        Self {
            foreground: Some(foreground),
            ..self
        }
    }
    pub const fn with_background(self, background: Rgb24) -> Self {
        Self {
            background: Some(background),
            ..self
        }
    }
    pub const fn without_bold(self) -> Self {
        Self { bold: None, ..self }
    }
    pub const fn without_underline(self) -> Self {
        Self {
            underline: None,
            ..self
        }
    }
    pub const fn without_foreground(self) -> Self {
        Self {
            foreground: None,
            ..self
        }
    }
    pub const fn without_background(self) -> Self {
        Self {
            background: None,
            ..self
        }
    }
    pub fn coalesce(self, other: Self) -> Self {
        Self {
            bold: (self.bold.or(other.bold)),
            underline: (self.underline.or(other.underline)),
            foreground: (self.foreground.or(other.foreground)),
            background: (self.background.or(other.background)),
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewCell {
    pub character: Option<char>,
    pub style: Style,
}

impl ViewCell {
    pub const fn new() -> Self {
        Self {
            character: None,
            style: Style::new(),
        }
    }
    pub const fn character(&self) -> Option<char> {
        self.character
    }
    pub const fn bold(&self) -> Option<bool> {
        self.style.bold
    }
    pub const fn underline(&self) -> Option<bool> {
        self.style.underline
    }
    pub const fn foreground(&self) -> Option<Rgb24> {
        self.style.foreground
    }
    pub const fn background(&self) -> Option<Rgb24> {
        self.style.background
    }
    pub const fn with_character(self, character: char) -> Self {
        Self {
            character: Some(character),
            ..self
        }
    }
    pub const fn with_bold(self, bold: bool) -> Self {
        Self {
            style: self.style.with_bold(bold),
            ..self
        }
    }
    pub const fn with_underline(self, underline: bool) -> Self {
        Self {
            style: self.style.with_underline(underline),
            ..self
        }
    }
    pub const fn with_foreground(self, foreground: Rgb24) -> Self {
        Self {
            style: self.style.with_foreground(foreground),
            ..self
        }
    }
    pub const fn with_background(self, background: Rgb24) -> Self {
        Self {
            style: self.style.with_background(background),
            ..self
        }
    }
    pub const fn without_character(self) -> Self {
        Self {
            character: None,
            ..self
        }
    }
    pub const fn without_bold(self) -> Self {
        Self {
            style: self.style.without_bold(),
            ..self
        }
    }
    pub const fn without_underline(self) -> Self {
        Self {
            style: self.style.without_underline(),
            ..self
        }
    }
    pub const fn without_foreground(self) -> Self {
        Self {
            style: self.style.without_foreground(),
            ..self
        }
    }
    pub const fn without_background(self) -> Self {
        Self {
            style: self.style.without_background(),
            ..self
        }
    }
    pub const fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }
    pub fn coalesce(self, other: Self) -> Self {
        Self {
            character: (self.character.or(other.character)),
            style: self.style.coalesce(other.style),
        }
    }
}
