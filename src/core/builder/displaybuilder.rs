use core::{Display, DisplayInfo};
use maths::Point2;

/// A display builder.
#[must_use]
pub struct DisplayBuilder {
    info: DisplayInfo,
}

pub fn create_displaybuilder() -> DisplayBuilder {
    DisplayBuilder {
        info: DisplayInfo { ..DisplayInfo::default() }
    }
}

impl DisplayBuilder {
    /// Sets a width for the display.
    pub fn width(mut self: Self, width: u32) -> Self {
        self.info.width = width;
        self
    }
    /// Sets a height for the display.
    pub fn height(mut self: Self, height: u32) -> Self {
        self.info.height = height;
        self
    }
    /// Sets dimensions for the display.
    pub fn dimensions<T>(mut self: Self, dimensions: T) -> Self where Point2<u32>: From<T> {
        let dimensions = Point2::<u32>::from(dimensions);
        self.info.width = dimensions.0;
        self.info.height = dimensions.1;
        self
    }
    /// Sets a title for the display.
    pub fn title(mut self: Self, title: &str) -> Self {
        self.info.title = title.to_string();
        self
    }
    /// Flags the display to be transparent.
    pub fn transparent(mut self: Self) -> Self {
        self.info.transparent = true;
        self
    }
    /// Flags the display to be borderless.
    pub fn borderless(mut self: Self) -> Self {
        self.info.decorations = false;
        self
    }
    /// Sets the monitor to create the display on.
    pub fn monitor(mut self: Self, id: i32) -> Self {
        self.info.monitor = id;
        self
    }
    /// Flags the display to use vsync.
    pub fn vsync(mut self: Self) -> Self {
        self.info.vsync = true;
        self
    }
    /// Returns the constructed display instance.
    pub fn build(self: Self) -> Display {
        Display::new(self.info)
    }
}
