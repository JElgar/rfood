pub trait PopFirst<T> {
    fn pop_first(&mut self) -> Option<T>;
}

impl<T, U> PopFirst<T> for syn::punctuated::Punctuated<T, U> where T: Clone, U: Default {
    fn pop_first(&mut self) -> Option<T> {
        let val = self.iter().next().cloned();
        *self = syn::punctuated::Punctuated::from_iter(self.iter().skip(1).cloned());
        val
    }
}

