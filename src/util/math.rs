use std::ops::{Add, AddAssign, BitXor, Div, DivAssign, Mul, MulAssign, Neg, Range, Rem, RemAssign, Sub, SubAssign};

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
}

macro_rules! consume_ident {
    ($type: ty, $i: ident) => { $type };
}

macro_rules! impl_vecn_base {
    ($struct_name: ident, $template_type: ident, $value_type: ty, $($x: ident),*) => {
        #[derive(Debug, Default, PartialEq)]
        pub struct $struct_name<$template_type> {
            $( pub $x : $value_type, )*
        }

        impl<$template_type: Clone> Clone for $struct_name<$template_type> where $value_type: Clone {
            fn clone(&self) -> Self {
                Self {
                    $( $x: self.$x.clone() ),*
                }
            }
        }

        impl<$template_type: Copy> Copy for $struct_name<$template_type> where $value_type: Copy {

        }

        impl<$template_type> $struct_name<$template_type> {
            pub fn new($($x: $value_type,)*) -> Self {
                Self { $($x,)* }
            }

            pub fn from_tuple(t: ( $( consume_ident!($value_type, $x) ),* )) -> Self {
                Self::from(t)
            }

            pub fn into_tuple(self) -> ( $( consume_ident!($value_type, $x) ),* ) {
                self.into()
            }
        }

        impl<$template_type> Into<( $( consume_ident!($value_type, $x) ),* )> for $struct_name<$template_type> {
            fn into(self) -> ( $( consume_ident!($value_type, $x) ),* ) {
                ( $( self.$x ),* )
            }
        }

        impl<$template_type> From<( $( consume_ident!($value_type, $x) ),* )> for $struct_name<$template_type> {
            fn from(t: ( $( consume_ident!($value_type, $x) ),* )) -> Self {
                let ($($x),*) = t;

                Self { $($x),* }
            }
        }
    }
}

macro_rules! impl_vecn_binary_operator {
    ($op_name: ident, $op_fn_name: ident, $struct_name: ident, $($x: ident),*) => {
        impl<A: $op_name<Output = A>> $op_name<$struct_name<A>> for $struct_name<A> {
            type Output = $struct_name<A>;

            fn $op_fn_name(self, rhs: $struct_name<A>) -> Self::Output {
                Self::Output {
                    $( $x: $op_name::$op_fn_name(self.$x, rhs.$x), )*
                }
            }
        }

        impl<T: Clone + $op_name<Output = T>> $op_name<T> for $struct_name<T> {
            type Output = $struct_name<T>;

            fn $op_fn_name(self, rhs: T) -> Self::Output {
                Self::Output {
                    $( $x: $op_name::$op_fn_name(self.$x, rhs.clone()), )*
                }
            }
        }
    }
}

macro_rules! impl_vecn_assignment_operator {
    ($op_name: ident, $op_fn_name: ident, $struct_name: ident, $($x: ident),*) => {
        impl<T: $op_name> $op_name<$struct_name<T>> for $struct_name<T> {
            fn $op_fn_name(&mut self, rhs: $struct_name<T>) {
                $( $op_name::<T>::$op_fn_name(&mut self.$x, rhs.$x); )*
            }
        }

        impl<T: Clone + $op_name> $op_name<T> for $struct_name<T> {
            fn $op_fn_name(&mut self, rhs: T) {
                $( $op_name::<T>::$op_fn_name(&mut self.$x, rhs.clone()); )*
            }
        }
    }
}

macro_rules! impl_vecn_unary_operator {
    ($op_name: ident, $op_fn_name: ident, $struct_name: ident, $($x: ident),*) => {
        impl<T: $op_name<Output = T>> $op_name for $struct_name<T> {
            type Output = $struct_name<T>;

            fn $op_fn_name(self) -> Self::Output {
                Self::Output {
                    $( $x: $op_name::$op_fn_name(self.$x), )*
                }
            }
        }
    }
}

macro_rules! operator_on_variadic {
    ($operator: tt, $first: expr) => {
        $first
    };

    ($operator: tt, $first: expr, $($rest: expr),*) => {
        $first $operator operator_on_variadic!($operator, $($rest),*)
    };
}

macro_rules! impl_vecn {
    ($struct_name: ident, $($x: ident),*) => {
        impl_vecn_base!($struct_name, T, T, $($x),*);

        impl<T: Add<T, Output = T> + Mul<T, Output = T>> BitXor for $struct_name<T> {
            type Output = T;

            fn bitxor(self, rhs: $struct_name<T>) -> Self::Output {
                operator_on_variadic!(+, $(self.$x * rhs.$x),*)
            }
        }

        impl<T: Add<T, Output = T> + Mul<T, Output = T> + Clone> $struct_name<T> {
            pub fn length2(&self) -> T {
                self.clone() ^ self.clone()
            }
        }

        impl<T: Add<T, Output = T> + Mul<T, Output = T> + Clone + numeric_traits::Sqrt> $struct_name<T> {
            pub fn length(&self) -> T {
                self.length2().sqrt()
            }
        }

        impl<T: Add<T, Output = T> + Mul<T, Output = T> + Div<T, Output = T> + Clone + numeric_traits::Sqrt> $struct_name<T> {
            pub fn normalized(&self) -> Self {
                let len = self.length();

                Self { $( $x: self.$x.clone() / len.clone() ),* }
            }

            pub fn normalize(&mut self) {
                let len = self.length();

                $( self.$x = self.$x.clone() / len.clone(); )*
            }
        }

        impl_vecn_binary_operator!(Add, add, $struct_name, $($x),*);
        impl_vecn_binary_operator!(Sub, sub, $struct_name, $($x),*);
        impl_vecn_binary_operator!(Mul, mul, $struct_name, $($x),*);
        impl_vecn_binary_operator!(Div, div, $struct_name, $($x),*);

        impl_vecn_unary_operator!(Neg, neg, $struct_name, $($x),*);

        impl_vecn_assignment_operator!(AddAssign, add_assign, $struct_name, $($x),*);
        impl_vecn_assignment_operator!(SubAssign, sub_assign, $struct_name, $($x),*);
        impl_vecn_assignment_operator!(MulAssign, mul_assign, $struct_name, $($x),*);
        impl_vecn_assignment_operator!(DivAssign, div_assign, $struct_name, $($x),*);
    }
}

macro_rules! impl_extn {
    ($struct_name: ident, $($x: ident),*) => {
        impl_vecn_base!($struct_name, T, T, $($x),*);
    }
}

macro_rules! impl_rectn {
    ($struct_name: ident, $point_name: ident, $ext_name: ident, $($x: ident),*) => {
        impl_vecn_base!($struct_name, T, Range<T>, $($x),*);

        impl<T> $struct_name<T> where Range<T>: ExactSizeIterator {
            pub fn extent(&self) -> $ext_name<usize> {
                $ext_name::<usize>::new($( self.$x.len() ),*)
            }
        }

        impl<T: Clone> $struct_name<T> {
            pub fn start(&self) -> $point_name<T> {
                $point_name::<T>::new($( self.$x.start.clone() ),*)
            }

            pub fn end(&self) -> $point_name<T> {
                $point_name::<T>::new($( self.$x.end.clone() ),*)
            }
        }
    }
}

impl_vecn!(Vec2, x, y);
impl_vecn!(Vec3, x, y, z);
impl_vecn!(Vec4, x, y, z, w);

impl_extn!(Ext2, w, h);
impl_extn!(Ext3, w, h, d);

impl_rectn!(Rect, Vec2, Ext2, x, y);
impl_rectn!(Box, Vec3, Ext3, x, y, z);

impl<T: Clone + Mul<T, Output = T> + Sub<T, Output = T>> Rem for Vec3<T> {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.y.clone() * rhs.z.clone() - self.z.clone() * rhs.y.clone(),
            y: self.z         * rhs.x.clone() - self.x.clone() * rhs.z,
            z: self.x         * rhs.y         - self.y         * rhs.x,
        }
    }
}

impl<T: Clone + Mul<T, Output = T> + Sub<T, Output = T>> RemAssign for Vec3<T> {
    fn rem_assign(&mut self, rhs: Self) {
        *self = self.clone() % rhs;
    }
}

impl<T: Clone + Mul<T, Output = T> + Sub<T, Output = T>> Rem for Vec2<T> {
    type Output = T;
    fn rem(self, rhs: Self) -> Self::Output {
        self.x * rhs.y - self.y * rhs.x
    }
}




pub struct Mat4x4<T> {
    pub data: [[T; 4]; 4],
}

impl<T: Clone> Clone for Mat4x4<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T: Copy> Copy for Mat4x4<T> {
}

impl<T: Mul<T, Output = T> + Add<T, Output = T> + Clone> Mul for Mat4x4<T> {
    type Output = Mat4x4<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            data: [
                [
                    self.data[0][0].clone() * rhs.data[0][0].clone() + self.data[0][1].clone() * rhs.data[1][0].clone() + self.data[0][2].clone() * rhs.data[2][0].clone() + self.data[0][3].clone() * rhs.data[3][0].clone(),
                    self.data[0][0].clone() * rhs.data[0][1].clone() + self.data[0][1].clone() * rhs.data[1][1].clone() + self.data[0][2].clone() * rhs.data[2][1].clone() + self.data[0][3].clone() * rhs.data[3][1].clone(),
                    self.data[0][0].clone() * rhs.data[0][2].clone() + self.data[0][1].clone() * rhs.data[1][2].clone() + self.data[0][2].clone() * rhs.data[2][2].clone() + self.data[0][3].clone() * rhs.data[3][2].clone(),
                    self.data[0][0].clone() * rhs.data[0][3].clone() + self.data[0][1].clone() * rhs.data[1][3].clone() + self.data[0][2].clone() * rhs.data[2][3].clone() + self.data[0][3].clone() * rhs.data[3][3].clone(),
                ],
                [
                    self.data[1][0].clone() * rhs.data[0][0].clone() + self.data[1][1].clone() * rhs.data[1][0].clone() + self.data[1][2].clone() * rhs.data[2][0].clone() + self.data[1][3].clone() * rhs.data[3][0].clone(),
                    self.data[1][0].clone() * rhs.data[0][1].clone() + self.data[1][1].clone() * rhs.data[1][1].clone() + self.data[1][2].clone() * rhs.data[2][1].clone() + self.data[1][3].clone() * rhs.data[3][1].clone(),
                    self.data[1][0].clone() * rhs.data[0][2].clone() + self.data[1][1].clone() * rhs.data[1][2].clone() + self.data[1][2].clone() * rhs.data[2][2].clone() + self.data[1][3].clone() * rhs.data[3][2].clone(),
                    self.data[1][0].clone() * rhs.data[0][3].clone() + self.data[1][1].clone() * rhs.data[1][3].clone() + self.data[1][2].clone() * rhs.data[2][3].clone() + self.data[1][3].clone() * rhs.data[3][3].clone(),
                ],
                [
                    self.data[2][0].clone() * rhs.data[0][0].clone() + self.data[2][1].clone() * rhs.data[1][0].clone() + self.data[2][2].clone() * rhs.data[2][0].clone() + self.data[2][3].clone() * rhs.data[3][0].clone(),
                    self.data[2][0].clone() * rhs.data[0][1].clone() + self.data[2][1].clone() * rhs.data[1][1].clone() + self.data[2][2].clone() * rhs.data[2][1].clone() + self.data[2][3].clone() * rhs.data[3][1].clone(),
                    self.data[2][0].clone() * rhs.data[0][2].clone() + self.data[2][1].clone() * rhs.data[1][2].clone() + self.data[2][2].clone() * rhs.data[2][2].clone() + self.data[2][3].clone() * rhs.data[3][2].clone(),
                    self.data[2][0].clone() * rhs.data[0][3].clone() + self.data[2][1].clone() * rhs.data[1][3].clone() + self.data[2][2].clone() * rhs.data[2][3].clone() + self.data[2][3].clone() * rhs.data[3][3].clone(),
                ],
                [
                    self.data[3][0].clone() * rhs.data[0][0].clone() + self.data[3][1].clone() * rhs.data[1][0].clone() + self.data[3][2].clone() * rhs.data[2][0].clone() + self.data[3][3].clone() * rhs.data[3][0].clone(),
                    self.data[3][0].clone() * rhs.data[0][1].clone() + self.data[3][1].clone() * rhs.data[1][1].clone() + self.data[3][2].clone() * rhs.data[2][1].clone() + self.data[3][3].clone() * rhs.data[3][1].clone(),
                    self.data[3][0].clone() * rhs.data[0][2].clone() + self.data[3][1].clone() * rhs.data[1][2].clone() + self.data[3][2].clone() * rhs.data[2][2].clone() + self.data[3][3].clone() * rhs.data[3][2].clone(),
                    self.data[3][0].clone() * rhs.data[0][3].clone() + self.data[3][1].clone() * rhs.data[1][3].clone() + self.data[3][2].clone() * rhs.data[2][3].clone() + self.data[3][3].clone() * rhs.data[3][3].clone(),
                ],
            ],
        }
    }
}

/// Pack of operators definition module
impl<T: Copy + Neg<Output = T> + Sub<T, Output = T> + Add<T, Output = T> + Mul<T, Output = T> + Div<T, Output = T>> Mat4x4<T> {
    /// Determinant getting function
    /// * Returns determinant of this matrix
    pub fn determinant(&self) -> T {
        self.data[0][0] * (self.data[1][1] * self.data[2][2] * self.data[3][3] + self.data[1][2] * self.data[2][3] * self.data[3][1] + self.data[1][3] * self.data[2][1] * self.data[3][2] - self.data[1][1] * self.data[2][3] * self.data[3][2] - self.data[1][2] * self.data[2][1] * self.data[3][3] - self.data[1][3] * self.data[2][2] * self.data[3][1]) -
        self.data[0][1] * (self.data[0][1] * self.data[2][2] * self.data[3][3] + self.data[0][2] * self.data[2][3] * self.data[3][1] + self.data[0][3] * self.data[2][1] * self.data[3][2] - self.data[0][1] * self.data[2][3] * self.data[3][2] - self.data[0][2] * self.data[2][1] * self.data[3][3] - self.data[0][3] * self.data[2][2] * self.data[3][1]) +
        self.data[0][2] * (self.data[0][1] * self.data[1][2] * self.data[3][3] + self.data[0][2] * self.data[1][3] * self.data[3][1] + self.data[0][3] * self.data[1][1] * self.data[3][2] - self.data[0][1] * self.data[1][3] * self.data[3][2] - self.data[0][2] * self.data[1][1] * self.data[3][3] - self.data[0][3] * self.data[1][2] * self.data[3][1]) -
        self.data[0][3] * (self.data[0][1] * self.data[1][2] * self.data[2][3] + self.data[0][2] * self.data[1][3] * self.data[2][1] + self.data[0][3] * self.data[1][1] * self.data[2][2] - self.data[0][1] * self.data[1][3] * self.data[2][2] - self.data[0][2] * self.data[1][1] * self.data[2][3] - self.data[0][3] * self.data[1][2] * self.data[2][1])
    } // fn determinant

    /// Matrix inversion getting function
    /// * Returns this matrix inersed
    pub fn inversed(&self) -> Self {
        let determ_00 = self.data[1][1] * self.data[2][2] * self.data[3][3] + self.data[1][2] * self.data[2][3] * self.data[3][1] + self.data[1][3] * self.data[2][1] * self.data[3][2] - self.data[1][1] * self.data[2][3] * self.data[3][2] - self.data[1][2] * self.data[2][1] * self.data[3][3] - self.data[1][3] * self.data[2][2] * self.data[3][1];
        let determ_01 = self.data[0][1] * self.data[2][2] * self.data[3][3] + self.data[0][2] * self.data[2][3] * self.data[3][1] + self.data[0][3] * self.data[2][1] * self.data[3][2] - self.data[0][1] * self.data[2][3] * self.data[3][2] - self.data[0][2] * self.data[2][1] * self.data[3][3] - self.data[0][3] * self.data[2][2] * self.data[3][1];
        let determ_02 = self.data[0][1] * self.data[1][2] * self.data[3][3] + self.data[0][2] * self.data[1][3] * self.data[3][1] + self.data[0][3] * self.data[1][1] * self.data[3][2] - self.data[0][1] * self.data[1][3] * self.data[3][2] - self.data[0][2] * self.data[1][1] * self.data[3][3] - self.data[0][3] * self.data[1][2] * self.data[3][1];
        let determ_03 = self.data[0][1] * self.data[1][2] * self.data[2][3] + self.data[0][2] * self.data[1][3] * self.data[2][1] + self.data[0][3] * self.data[1][1] * self.data[2][2] - self.data[0][1] * self.data[1][3] * self.data[2][2] - self.data[0][2] * self.data[1][1] * self.data[2][3] - self.data[0][3] * self.data[1][2] * self.data[2][1];

        let determ =
            self.data[0][0] * determ_00 -
            self.data[0][1] * determ_01 +
            self.data[0][2] * determ_02 -
            self.data[0][3] * determ_03;

        Self {
            data: [
                [
                     self.data[0][0] * determ_00 / determ,
                    -self.data[0][1] * determ_01 / determ,
                     self.data[0][2] * determ_02 / determ,
                    -self.data[0][3] * determ_03 / determ,
                ],
                [
                    -self.data[1][0] * (self.data[0][1] * self.data[2][2] * self.data[3][3] + self.data[0][2] * self.data[2][3] * self.data[3][1] + self.data[0][3] * self.data[2][1] * self.data[3][2] - self.data[0][1] * self.data[2][3] * self.data[3][2] - self.data[0][2] * self.data[2][1] * self.data[3][3] - self.data[0][3] * self.data[2][2] * self.data[3][1]) / determ,
                     self.data[1][1] * (self.data[0][0] * self.data[2][2] * self.data[3][3] + self.data[0][2] * self.data[2][3] * self.data[3][0] + self.data[0][3] * self.data[2][0] * self.data[3][2] - self.data[0][0] * self.data[2][3] * self.data[3][2] - self.data[0][2] * self.data[2][0] * self.data[3][3] - self.data[0][3] * self.data[2][2] * self.data[3][0]) / determ,
                    -self.data[1][2] * (self.data[0][0] * self.data[2][1] * self.data[3][3] + self.data[0][1] * self.data[2][3] * self.data[3][0] + self.data[0][3] * self.data[2][0] * self.data[3][1] - self.data[0][0] * self.data[2][3] * self.data[3][1] - self.data[0][1] * self.data[2][0] * self.data[3][3] - self.data[0][3] * self.data[2][1] * self.data[3][0]) / determ,
                     self.data[1][3] * (self.data[0][0] * self.data[2][1] * self.data[3][2] + self.data[0][1] * self.data[2][2] * self.data[3][0] + self.data[0][2] * self.data[2][0] * self.data[3][1] - self.data[0][0] * self.data[2][2] * self.data[3][1] - self.data[0][1] * self.data[2][0] * self.data[3][2] - self.data[0][2] * self.data[2][1] * self.data[3][0]) / determ,
                ],
                [
                     self.data[2][0] * (self.data[0][1] * self.data[1][2] * self.data[3][3] + self.data[0][2] * self.data[1][3] * self.data[3][1] + self.data[0][3] * self.data[1][1] * self.data[3][2] - self.data[0][1] * self.data[1][3] * self.data[3][2] - self.data[0][2] * self.data[1][1] * self.data[3][3] - self.data[0][3] * self.data[1][2] * self.data[3][1]) / determ,
                    -self.data[2][1] * (self.data[0][0] * self.data[1][2] * self.data[3][3] + self.data[0][2] * self.data[1][3] * self.data[3][0] + self.data[0][3] * self.data[1][0] * self.data[3][2] - self.data[0][0] * self.data[1][3] * self.data[3][2] - self.data[0][2] * self.data[1][0] * self.data[3][3] - self.data[0][3] * self.data[1][2] * self.data[3][0]) / determ,
                     self.data[2][2] * (self.data[0][0] * self.data[1][1] * self.data[3][3] + self.data[0][1] * self.data[1][3] * self.data[3][0] + self.data[0][3] * self.data[1][0] * self.data[3][1] - self.data[0][0] * self.data[1][3] * self.data[3][1] - self.data[0][1] * self.data[1][0] * self.data[3][3] - self.data[0][3] * self.data[1][1] * self.data[3][0]) / determ,
                    -self.data[2][3] * (self.data[0][0] * self.data[1][1] * self.data[3][2] + self.data[0][1] * self.data[1][2] * self.data[3][0] + self.data[0][2] * self.data[1][0] * self.data[3][1] - self.data[0][0] * self.data[1][2] * self.data[3][1] - self.data[0][1] * self.data[1][0] * self.data[3][2] - self.data[0][2] * self.data[1][1] * self.data[3][0]) / determ,
                ],
                [
                    -self.data[3][0] * (self.data[0][1] * self.data[1][2] * self.data[2][3] + self.data[0][2] * self.data[1][3] * self.data[2][1] + self.data[0][3] * self.data[1][1] * self.data[2][2] - self.data[0][1] * self.data[1][3] * self.data[2][2] - self.data[0][2] * self.data[1][1] * self.data[2][3] - self.data[0][3] * self.data[1][2] * self.data[2][1]) / determ,
                     self.data[3][1] * (self.data[0][0] * self.data[1][2] * self.data[2][3] + self.data[0][2] * self.data[1][3] * self.data[2][0] + self.data[0][3] * self.data[1][0] * self.data[2][2] - self.data[0][0] * self.data[1][3] * self.data[2][2] - self.data[0][2] * self.data[1][0] * self.data[2][3] - self.data[0][3] * self.data[1][2] * self.data[2][0]) / determ,
                    -self.data[3][2] * (self.data[0][0] * self.data[1][1] * self.data[2][3] + self.data[0][1] * self.data[1][3] * self.data[2][0] + self.data[0][3] * self.data[1][0] * self.data[2][1] - self.data[0][0] * self.data[1][3] * self.data[2][1] - self.data[0][1] * self.data[1][0] * self.data[2][3] - self.data[0][3] * self.data[1][1] * self.data[2][0]) / determ,
                     self.data[3][3] * (self.data[0][0] * self.data[1][1] * self.data[2][2] + self.data[0][1] * self.data[1][2] * self.data[2][0] + self.data[0][2] * self.data[1][0] * self.data[2][1] - self.data[0][0] * self.data[1][2] * self.data[2][1] - self.data[0][1] * self.data[1][0] * self.data[2][2] - self.data[0][2] * self.data[1][1] * self.data[2][0]) / determ,
                ],
           ]
        }
    } // fn inversed
} // impl<T: Copy + Neg<Output = T> + Sub<T, Output = T> + Add<T, Output = T> + Mul<T, Output = T> + Div<T, Output = T>> Mat4x4<T>



/// Default matrices implementation
impl Mat4x4<f32> {
    /// Identity matrix getting function
    /// * Returns identity matrix
    pub const fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    } // fn identity

    /// Rotation matrix getting function
    /// * `angle` - angle to create rotation matrix on
    /// * `axis` - axis to create rotation matrix based on
    /// * Returns rotation matrix
    pub fn rotate(angle: f32, mut axis: Vec3<f32>) -> Self {
        axis.normalize();

        let sina = angle.sin();
        let cosa = angle.cos();

        Self {
            data: [
                [axis.x * axis.x * (1.0 - cosa) + cosa,            axis.x * axis.y * (1.0 - cosa) - axis.z * sina,   axis.x * axis.z * (1.0 - cosa) + axis.y * sina,   0.0],
                [axis.y * axis.x * (1.0 - cosa) + axis.z * sina,   axis.y * axis.y * (1.0 - cosa) + cosa,            axis.y * axis.z * (1.0 - cosa) - axis.x * sina,   0.0],
                [axis.z * axis.x * (1.0 - cosa) - axis.y * sina,   axis.z * axis.y * (1.0 - cosa) + axis.x * sina,   axis.z * axis.z * (1.0 - cosa) + cosa,            0.0],
                [0.0,                                              0.0,                                              0.0,                                              1.0]
            ]
        }
    } // fn rotate

    /// Rotation matrix getting function
    /// * `angle` - angle to create rotation matrix on
    /// * Returns rotation matrix
    pub fn rotate_x(angle: f32) -> Self
    {
        let sin = angle.sin();
        let cos = angle.cos();

        Self {
            data: [
                [1.0,  0.0, 0.0, 0.0],
                [0.0,  cos, sin, 0.0],
                [0.0, -sin, cos, 0.0],
                [0.0,  0.0, 0.0, 1.0]
            ]
        }
    } // fn rotate_x

    /// Rotation matrix getting function
    /// * `angle` - angle to create rotation matrix on
    /// * Returns rotation matrix
    pub fn rotate_y(angle: f32) -> Self
    {

        let sin = angle.sin();
        let cos = angle.cos();

        Self {
            data: [
                [cos, 0.0, -sin, 0.0],
                [0.0, 1.0,  0.0, 0.0],
                [sin, 0.0,  cos, 0.0],
                [0.0, 0.0,  0.0, 1.0],
            ]
        }
    } // fn rotate_y

    /// Rotation matrix getting function
    /// * `angle` - angle to create rotation matrix on
    /// * Returns rotation matrix
    pub fn rotate_z(angle: f32) -> Self
    {
        let sin = angle.sin();
        let cos = angle.cos();

        Self {
            data: [
                [ cos, sin, 0.0, 0.0],
                [-sin, cos, 0.0, 0.0],
                [ 0.0, 0.0, 1.0, 0.0],
                [ 0.0, 0.0, 0.0, 1.0],
            ]
        }
    } // fn rotate_z

    /// Scaling function.
    /// * `s` - scale vector
    /// * Returns scale matrix
    pub fn scale(s: Vec3<f32>) -> Self {
        Self {
            data: [
                [s.x, 0.0, 0.0, 0.0],
                [0.0, s.y, 0.0, 0.0],
                [0.0, 0.0, s.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    } // fn scale

    /// Translating function.
    /// * `t` - translate vector
    /// * Returns scale matrix
    pub fn translate(t: Vec3<f32>) -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [t.x, t.y, t.z, 1.0],
            ]
        }
    } // fn translate
} // impl Mat4x4<f32>

/// Projection functions implementation
impl Mat4x4<f32> {
    /// Orthographic projection matrix create function
    /// * `l`, `r` - left and right boundaries
    /// * `b`, `t` - bottom and top
    /// * `n`, `f` - near and far
    /// * Returns projection matrix
    pub fn projection_ortho(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4x4<f32> {
        Self {
            data: [
                [2.0 / (r - l),      0.0,                0.0,                0.0],
                [0.0,                2.0 / (t - b),      0.0,                0.0],
                [0.0,                0.0,                -2.0 / (f - n),     0.0],
                [-(r + l) / (r - l), -(t + b) / (t - b), -(f + n) / (f - n), 1.0]
            ]
        }
    } // fn projection_ortho

    pub fn projection_frustum(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4x4<f32> {
        Self {
            data: [
                [2.0 * n / (r - l),   0.0,                  0.0,                   0.0],
                [0.0,                 2.0 * n / (t - b),    0.0,                   0.0],
                [(r + l) / (r - l),   (t + b) / (t - b),   -(f + n) / (f - n),    -1.0],
                [0.0,                 0.0,                 -2.0 * n * f / (f - n), 0.0]
            ]
        }
    } // fn projection_frustum

    pub fn view(loc: Vec3<f32>, at: Vec3<f32>, approx_up: Vec3<f32>) -> Mat4x4<f32> {
        let dir = (at - loc).normalized();
        let right = (dir % approx_up).normalized();
        let up = (right % dir).normalized();

        Self {
            data: [
                [right.x,         up.x,        -dir.x,    0.0],
                [right.y,         up.y,        -dir.y,    0.0],
                [right.z,         up.z,        -dir.z,    0.0],
                [-(loc ^ right),  -(loc ^ up), loc ^ dir, 1.0],
            ]
        }
    } // fn view
} // impl Mat4x4<f32>

impl Mat4x4<f32> {
    pub fn transform_vector(&self, v: Vec3<f32>) -> Vec3<f32> {
        Vec3 {
            x: v.x * self.data[0][0] + v.y * self.data[1][0] + v.z * self.data[2][0],
            y: v.x * self.data[0][1] + v.y * self.data[1][1] + v.z * self.data[2][1],
            z: v.x * self.data[0][2] + v.y * self.data[1][2] + v.z * self.data[2][2],
        }
    } // fn transform_vector

    pub fn transform_point(&self, v: Vec3<f32>) -> Vec3<f32> {
        Vec3 {
            x: v.x * self.data[0][0] + v.y * self.data[1][0] + v.z * self.data[2][0] + self.data[3][0],
            y: v.x * self.data[0][1] + v.y * self.data[1][1] + v.z * self.data[2][1] + self.data[3][1],
            z: v.x * self.data[0][2] + v.y * self.data[1][2] + v.z * self.data[2][2] + self.data[3][2],
        }
    } // fn transform_point

    pub fn transform_4x4(&self, v: Vec3<f32>) -> Vec3<f32> {
        let w = v.x * self.data[0][3] + v.y * self.data[1][3] + v.z * self.data[2][3] + self.data[3][3];

        Vec3 {
            x: (v.x * self.data[0][0] + v.y * self.data[1][0] + v.z * self.data[2][0] + self.data[3][0]) / w,
            y: (v.x * self.data[0][1] + v.y * self.data[1][1] + v.z * self.data[2][1] + self.data[3][1]) / w,
            z: (v.x * self.data[0][2] + v.y * self.data[1][2] + v.z * self.data[2][2] + self.data[3][2]) / w,
        }
    } // fn transform_4x4
} // impl Mat4x4
