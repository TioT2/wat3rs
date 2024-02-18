/// WAT3RS Project
/// `File` util/math/vec.rs
/// `Description` Math main declaration module
/// `Author` TioT2
/// `Last changed` 18.02.2024

use super::vecn::vecn;

// Default n-component vector declarations
vecn!(Vec2, x, y);
vecn!(Vec3, x, y, z);
vecn!(Vec4, x, y, z, w);

/// 3-component vector cross product implementation
impl<T: Copy + core::ops::Mul<T, Output = T> + core::ops::Sub<T, Output = T>> core::ops::Rem<Vec3<T>> for Vec3<T> {
    type Output = Vec3<T>;

    /// Cross product getting operator
    /// * `rhs` - vector to get cross product with
    /// * Returns vectors cross product
    fn rem(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    } // fn rem
} // impl<T: Copy + core::ops::Mul<T, Output = T> + core::ops::Sub<T, Output = T>> core::ops::Rem<Vec3<T>> for Vec3<T>

// file vec.rs
