//The repr here causes the entire struct to use the layout of the chars field

#[repr(transparent)]
pub struct ScreenBuffer<Unit, const WIDTH: usize, const HEIGHT: usize> {
    pub chars: [[Unit; WIDTH]; HEIGHT],
}

//implements width and height functions for all screenbuffers, any possible combo.
impl<Unit, const WIDTH: usize, const HEIGHT: usize> ScreenBuffer<Unit, WIDTH, HEIGHT> {
    pub const BUFFER_WIDTH: usize = WIDTH;
    pub const BUFFER_HEIGHT: usize = HEIGHT;
    /// Gathers the width of screen buffer
    #[inline(always)]
    pub const fn width(&self) -> usize {
        WIDTH
    }

    /// Gathers the height of screen buffer
    #[inline(always)]
    pub const fn height(&self) -> usize {
        HEIGHT
    }
}
