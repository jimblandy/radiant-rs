use core::*;

/// A Texture builder.
#[must_use]
#[derive(Clone)]
pub struct TextureBuilder<'a> {
    info    : TextureInfo<'a>,
    context : &'a RenderContext,
    //file    : Option<&'a str>,
}

impl<'a> TextureBuilder<'a> {
    /// Sets a width for the texture.
    pub fn width(mut self: Self, width: u32) -> Self {
        self.info.width = width;
        self
    }
    /// Sets a height for the texture.
    pub fn height(mut self: Self, height: u32) -> Self {
        self.info.height = height;
        self
    }
    /// Sets dimensions for the texture.
    pub fn dimensions<T>(mut self: Self, dimensions: T) -> Self where Point2<u32>: From<T> {
        let dimensions = Point2::<u32>::from(dimensions);
        self.info.width = dimensions.0;
        self.info.height = dimensions.1;
        self
    }
    /// Sets a minification filter for the texture.
    pub fn minify(mut self: Self, minify: TextureFilter) -> Self {
        self.info.minify = minify;
        self
    }
    /// Sets a magnification filter for the texture.
    pub fn magnify(mut self: Self, magnify: TextureFilter) -> Self {
        self.info.magnify = magnify;
        self
    }
    /// Sets a wrapping type for the texture.
    pub fn wrap(mut self: Self, wrap: TextureWrap) -> Self {
        self.info.wrap = wrap;
        self
    }
    /// Sets an internal format for the texture.
    pub fn format(mut self: Self, format: TextureFormat) -> Self {
        self.info.format = format;
        self
    }
    pub fn file(mut self: Self, file: &'a str) -> Self {
        self.info.file = Some(file);
        self
    }
    /// Returns the constructed Texture instance.
    pub fn build(self: Self) -> Result<Texture> {
        Texture::from_info(self.context, self.info)
    }
    pub(crate) fn new<'b>(context: &'b RenderContext) -> TextureBuilder {
        TextureBuilder {
            context : context,
            info    : TextureInfo { ..TextureInfo::default() },
            //file    : None,
        }
    }
}
