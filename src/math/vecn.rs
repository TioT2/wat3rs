/// WAT3RS Project
/// `File` util/math/vecn.rs
/// `Description` N-component vector template implementaiton module
/// `Author` TioT2
/// `Last changed` 18.02.2024

/// N-component vector declaration macro
macro_rules! declare_struct {
    ($type_name: ident, $($x: ident),* ) => {
        pub struct $type_name<T> {
            $( pub $x : T, )*
        }
    }
} // macro_rules! vecn_declare_struct

macro_rules! impl_basic {
    ($type: ident, $($x: ident),*) => {
        impl<T: Clone> $type<T> {
            pub fn new($($x: T, )*) -> $type<T> {
                $type::<T> {$($x: $x.clone(), )*} }
        }

        impl<T: Clone> Clone for $type<T> {
            fn clone(&self) -> $type<T> {
                $type::<T> {$($x: self.$x.clone(), )*}
            }
        }

        impl<T: Copy> Copy for $type<T> {
        }

        impl<T: Default> Default for $type<T> {
            fn default() -> Self {
                $type::<T> {$($x: T::default(), )*}
            }
        }
    }
}

// Implementation for +-*/ operators macro redefinition
macro_rules! impl_operator {
    ($op_name: ident, $op_fn_name: ident, $type: ident, $($x: ident),*) => {
        /* Vec2<T> op Vec2<T> */
        impl<T: Copy + Clone + core::ops::$op_name<Output = T>> core::ops::$op_name<$type<T>> for $type<T> {
            type Output = $type<T>;
            fn $op_fn_name(self, rhs: $type<T>) -> $type<T> {
                $type::<T> {
                    $( $x: core::ops::$op_name::<T>::$op_fn_name(self.$x, rhs.$x) ),*
                }
            }
        }

        /* Vec2<T> op T */
        impl<T: Copy + Clone + core::ops::$op_name<Output = T>> core::ops::$op_name<T> for $type<T> {
            type Output = $type<T>;
            fn $op_fn_name(self, rhs: T) -> $type<T> {
                $type::<T> {
                    $( $x: core::ops::$op_name::<T>::$op_fn_name(self.$x, rhs) ),*
                }
            }
        }
    }
} /* macro_rules! vec2_impl_operator */

macro_rules! impl_assign_operator {
    ($op_name: ident, $op_fn_name: ident, $type: ident, $($x: ident),*) => {
        impl<T: Copy + Clone + core::ops::$op_name<T>> core::ops::$op_name<$type<T>> for $type<T> {
            fn $op_fn_name(&mut self, rhs: $type<T>) {
                $( core::ops::$op_name::<T>::$op_fn_name(&mut self.$x, rhs.$x); )*
            }
        }

        impl<T: Copy + Clone + core::ops::$op_name<T>> core::ops::$op_name<T> for $type<T> {
            fn $op_fn_name(&mut self, rhs: T) {
                $( core::ops::$op_name::<T>::$op_fn_name(&mut self.$x, rhs); )*
            }
        }
    }
} /* macro_rules! impl_assign_operator */

macro_rules! impl_neg_operator {
    ($type: ident, $($x: ident),*) => {
        impl<T: Copy + Clone + core::ops::Neg<Output = T>> core::ops::Neg for $type<T> {
            type Output = $type<T>;
            fn neg(self) -> $type<T> {
                $type::<T> { $($x: -self.$x,)* }
            }
        }
    }
}

macro_rules! operator_on_variadic {
    ($operator: tt, $first: expr) => {
        $first
    };

    ($operator: tt, $first: expr, $($rest: expr),*) => {
        $first $operator crate::math::vecn::operator_on_variadic!($operator, $($rest),*)
    };
}

macro_rules! impl_dot_operator {
    ($type: ident, $($x: ident),*) => {
        impl<T: Copy + Clone + core::ops::Mul<T, Output = T> + core::ops::Add<T, Output = T>> $type<T> {
            pub fn dot(&self, rhs: $type<T>) -> T {
                crate::math::vecn::operator_on_variadic!(+, $(self.$x * rhs.$x),*)
            }
        }

        impl<T: Copy + Clone + core::ops::Mul<T, Output = T> + core::ops::Add<T, Output = T>> core::ops::BitXor<$type<T>> for $type<T> {
            type Output = T;
            fn bitxor(self, rhs: $type<T>) -> T {
                crate::math::vecn::operator_on_variadic!(+, $(self.$x * rhs.$x),*)
            }
        }
    }
}

macro_rules! impl_length_normalize {
    ($fxx: ident, $type: ident, $($x: ident),*) => {
        impl $type<$fxx> {
            pub fn length2(&self) -> $fxx {
                crate::math::vecn::operator_on_variadic!(+, $(self.$x * self.$x),*)
            }

            pub fn length(&self) -> $fxx {
                (crate::math::vecn::operator_on_variadic!(+, $(self.$x * self.$x),*)).sqrt()
            }

            pub fn normalize(&mut self) -> &mut $type<$fxx> {
                let length = self.length();
                $(self.$x /= length;)*
                return self;
            }

            pub fn normalized(&self) -> $type<$fxx> {
                let length = self.length();
                $type::<$fxx> {
                    $($x: self.$x / length),*
                }
            }
        }
    }
}

/// N-component vector with following functionality declaration module
macro_rules! vecn {
    ($type: ident, $($x: ident),*) => {
        crate::math::vecn::declare_struct!($type, $($x),*);
        crate::math::vecn::impl_basic!($type, $($x),*);

        crate::math::vecn::impl_neg_operator!($type, $($x),*);
        crate::math::vecn::impl_dot_operator!($type, $($x),*);

        crate::math::vecn::impl_operator!(Add, add, $type, $($x),*);
        crate::math::vecn::impl_operator!(Sub, sub, $type, $($x),*);
        crate::math::vecn::impl_operator!(Mul, mul, $type, $($x),*);
        crate::math::vecn::impl_operator!(Div, div, $type, $($x),*);

        crate::math::vecn::impl_assign_operator!(AddAssign, add_assign, $type, $($x),*);
        crate::math::vecn::impl_assign_operator!(SubAssign, sub_assign, $type, $($x),*);
        crate::math::vecn::impl_assign_operator!(MulAssign, mul_assign, $type, $($x),*);
        crate::math::vecn::impl_assign_operator!(DivAssign, div_assign, $type, $($x),*);

        crate::math::vecn::impl_length_normalize!(f32, $type, $($x),*);
        crate::math::vecn::impl_length_normalize!(f64, $type, $($x),*);
    }
}

pub(crate) use impl_assign_operator;
pub(crate) use impl_length_normalize;
pub(crate) use impl_operator;
pub(crate) use impl_neg_operator;
pub(crate) use impl_dot_operator;
pub(crate) use impl_basic;
pub(crate) use operator_on_variadic;
pub(crate) use declare_struct;

pub(crate) use vecn;

// file vecn.rs
