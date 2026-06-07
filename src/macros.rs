// General purpose macros

/// Implement `Deref` and `DerefMut` on a newtype,
/// so that you can use `x.y` instead of `x.0.y`.
#[macro_export]
macro_rules! newtype {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident($field_vis:vis $inner:ty);
    ) => {
        $(#[$meta])*
        $vis struct $name($field_vis $inner);

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}
