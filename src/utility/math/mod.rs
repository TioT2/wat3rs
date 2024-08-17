pub use mat4x4::*;
pub use quat::*;
pub use vec::*;

mod mat4x4;
mod quat;
mod vec;

pub mod numeric_traits {
    pub trait Sqrt {
        fn sqrt(self) -> Self;
    }

    impl Sqrt for f32 {
        fn sqrt(self) -> Self {
            self.sqrt()
        }
    }

    impl Sqrt for f64 {
        fn sqrt(self) -> Self {
            self.sqrt()
        }
    }

    pub trait SinCos {
        fn sin(self) -> Self;
        fn cos(self) -> Self;
    }

    impl SinCos for f32 {
        fn sin(self) -> Self {
            self.sin()
        }

        fn cos(self) -> Self {
            self.cos()
        }
    }

    impl SinCos for f64 {
        fn sin(self) -> Self {
            self.sin()
        }

        fn cos(self) -> Self {
            self.cos()
        }
    }
}
