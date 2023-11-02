use std::path::{PathBuf, Path};

use tiny_skia::Pixmap;




pub enum Source {
    Mazefile {
        input: std::path::PathBuf
    },
    FromInputMask {
        input: std::path::PathBuf
    },
    Unmasked {
        width: usize,
        height: usize,
    },
    UnmaskedRadial {
        starting_branch_count: usize,
        ring_count: usize,
    },
}

pub enum Destination {
    Mazefile {
        output: PathBuf
    },
    Image {
        output: PathBuf,
        image_width: usize,
        padding: usize,
    }
}

pub struct Command {
    pub destination: Destination,

    pub source: Source,
}

pub struct CommandBuilder {
    b_destination: Option<Destination>,
    b_source: Option<Source>,
}

impl Source {
    pub fn mazefile(input: impl Into<PathBuf>) -> Self {
        Self::Mazefile{ input: input.into() }
    }

    pub fn input_mask(input: impl Into<PathBuf>) -> Self {
        Self::FromInputMask { input: input.into() }
    }

    pub fn unmasked(width: usize, height: usize) -> Self {
        Self::Unmasked { width, height }
    }

    pub fn unmasked_radial(starting_branch_count: usize, rings: usize) -> Self {
        Self::UnmaskedRadial { starting_branch_count, ring_count: rings }
    }
}

impl Destination {
    pub fn image(image_width: usize, padding: usize, output: impl Into<PathBuf>) -> Self {
        Self::Image{ image_width, padding, output: output.into() }
    }

    pub fn mazefile(output: impl Into<PathBuf>) -> Self {
        Self::Mazefile{ output: output.into() }
    }
}

impl CommandBuilder {
    pub fn new() -> Self {
        CommandBuilder { b_destination: None, b_source: None }
    }

    pub fn destination(mut self, destination: Destination) -> Self {
        self.b_destination = Some(destination);
        self
    }

    pub fn source(mut self, source: Source) -> Self {
        self.b_source = Some(source);
        self
    }

    pub fn build(self) -> Option<Command> {
        Some(Command {
            destination: self.b_destination?,
            source: self.b_source?
        })
    }
}