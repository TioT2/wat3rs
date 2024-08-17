use std::ops::{Add, BitXor, Div, Mul, Neg, Sub};

use super::{numeric_traits, Vec3};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quat<T> {
    pub s: T,
    pub v: Vec3<T>,
}

impl<T> Quat<T> {
    pub fn new(s: T, v: Vec3<T>) -> Self {
        Self { s, v }
    }
}

impl<T: Default> Quat<T> {
    pub fn real(s: T) -> Self {
        Self {
            s,
            v: Default::default(),
        }
    }

    pub fn pure(v: Vec3<T>) -> Self {
        Self {
            s: Default::default(),
            v,
        }
    }
}

impl<
        T: From<f32>
            + numeric_traits::SinCos
            + numeric_traits::Sqrt
            + Add<T, Output = T>
            + Mul<T, Output = T>
            + Div<T, Output = T>
            + Clone,
    > Quat<T>
{
    pub fn rotation(mut angle: T, axis: Vec3<T>) -> Self {
        angle = angle * 0.5.into();

        Self {
            s: angle.clone().cos(),
            v: axis.normalized() * angle.sin(),
        }
    }
}

impl<T: Add<T, Output = T>> Add<Quat<T>> for Quat<T> {
    type Output = Quat<T>;
    fn add(self, rhs: Quat<T>) -> Self::Output {
        Self::Output {
            s: self.s + rhs.s,
            v: self.v + rhs.v,
        }
    }
}

impl<T: Neg<Output = T>> Quat<T> {
    pub fn conjugate(self) -> Self {
        Self {
            s: self.s,
            v: -self.v,
        }
    }
}

impl<T: numeric_traits::Sqrt + Clone + Add<T, Output = T> + Mul<T, Output = T>> Quat<T> {
    pub fn norm2(self) -> T {
        self.s.clone() * self.s + (self.v.clone() ^ self.v)
    }

    pub fn norm(self) -> T {
        (self.s.clone() * self.s + (self.v.clone() ^ self.v)).sqrt()
    }
}

impl<
        T: numeric_traits::Sqrt
            + Clone
            + Div<T, Output = T>
            + Add<T, Output = T>
            + Mul<T, Output = T>
            + Neg<Output = T>,
    > Quat<T>
{
    pub fn normalized(self) -> Self {
        let norm = (self.s.clone() * self.s.clone() + (self.v.clone() ^ self.v.clone())).sqrt();

        Self {
            s: self.s / norm.clone(),
            v: self.v / norm,
        }
    }

    pub fn inversed(self) -> Self {
        let norm = self.s.clone() * self.s.clone() + (self.v.clone() ^ self.v.clone());

        Self {
            s: self.s / norm.clone(),
            v: self.v / -norm,
        }
    }
}

impl<
        T: numeric_traits::Sqrt
            + Clone
            + Div<T, Output = T>
            + Add<T, Output = T>
            + Sub<T, Output = T>
            + Mul<T, Output = T>
            + Neg<Output = T>
            + Default,
    > Quat<T>
{
    pub fn rotate_vector(self, v: Vec3<T>) -> Vec3<T> {
        (self.clone() * Quat::<T>::pure(v) * self.inversed()).v
    }
}

impl<T: Add<T, Output = T> + Mul<T, Output = T>> BitXor<Quat<T>> for Quat<T> {
    type Output = T;
    fn bitxor(self, rhs: Quat<T>) -> Self::Output {
        self.s * rhs.s + (self.v ^ rhs.v)
    }
}

impl<T: Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T> + Clone> Mul<Quat<T>>
    for Quat<T>
{
    type Output = Quat<T>;

    fn mul(self, rhs: Quat<T>) -> Self::Output {
        Self::Output {
            s: self.s.clone() * rhs.s.clone() - (self.v.clone() ^ rhs.v.clone()),
            v: Vec3 {
                x: self.s.clone() * rhs.v.x.clone()
                    + rhs.s.clone() * self.v.x.clone()
                    + self.v.y.clone() * rhs.v.z.clone()
                    - self.v.z.clone() * rhs.v.y.clone(),
                y: self.s.clone() * rhs.v.y.clone()
                    + rhs.s.clone() * self.v.y.clone()
                    + self.v.z.clone() * rhs.v.x.clone()
                    - self.v.x.clone() * rhs.v.z.clone(),
                z: self.s.clone() * rhs.v.z.clone()
                    + rhs.s.clone() * self.v.z.clone()
                    + self.v.x.clone() * rhs.v.y.clone()
                    - self.v.y.clone() * rhs.v.x.clone(),
            },
        }
    }
}

impl<T: Mul<T, Output = T> + Clone> Mul<T> for Quat<T> {
    type Output = Quat<T>;
    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            s: self.s * rhs.clone(),
            v: self.v * rhs,
        }
    }
}
