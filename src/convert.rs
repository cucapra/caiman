pub trait ConversionContext {
    type Error: std::error::Error;
}
pub trait Convert<T, C: ConversionContext> {
    fn convert(self, context: &C) -> Result<T, C::Error>;
}

// Blanket impl for conversions that don't convert anything
impl<T, C: ConversionContext> Convert<T, C> for T {
    #[inline]
    fn convert(self, _: &C) -> Result<T, C::Error> {
        Ok(self)
    }
}
