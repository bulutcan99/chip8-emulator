use super::window::CustomWindow;

pub struct Controller<'a> {
    window: &'a CustomWindow<'a>,
}

impl<'a> Controller<'a> {
    pub fn new(window: &'a CustomWindow<'a>) -> Self {
        Self { window }
    }
}
