use super::Vec3;
use std::ops::{Add, Div, Mul, Neg, Sub};

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

impl<T: Copy> Copy for Mat4x4<T> {}

impl<T: Mul<T, Output = T> + Add<T, Output = T> + Clone> Mul for Mat4x4<T> {
    type Output = Mat4x4<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            data: [
                [
                    self.data[0][0].clone() * rhs.data[0][0].clone()
                        + self.data[0][1].clone() * rhs.data[1][0].clone()
                        + self.data[0][2].clone() * rhs.data[2][0].clone()
                        + self.data[0][3].clone() * rhs.data[3][0].clone(),
                    self.data[0][0].clone() * rhs.data[0][1].clone()
                        + self.data[0][1].clone() * rhs.data[1][1].clone()
                        + self.data[0][2].clone() * rhs.data[2][1].clone()
                        + self.data[0][3].clone() * rhs.data[3][1].clone(),
                    self.data[0][0].clone() * rhs.data[0][2].clone()
                        + self.data[0][1].clone() * rhs.data[1][2].clone()
                        + self.data[0][2].clone() * rhs.data[2][2].clone()
                        + self.data[0][3].clone() * rhs.data[3][2].clone(),
                    self.data[0][0].clone() * rhs.data[0][3].clone()
                        + self.data[0][1].clone() * rhs.data[1][3].clone()
                        + self.data[0][2].clone() * rhs.data[2][3].clone()
                        + self.data[0][3].clone() * rhs.data[3][3].clone(),
                ],
                [
                    self.data[1][0].clone() * rhs.data[0][0].clone()
                        + self.data[1][1].clone() * rhs.data[1][0].clone()
                        + self.data[1][2].clone() * rhs.data[2][0].clone()
                        + self.data[1][3].clone() * rhs.data[3][0].clone(),
                    self.data[1][0].clone() * rhs.data[0][1].clone()
                        + self.data[1][1].clone() * rhs.data[1][1].clone()
                        + self.data[1][2].clone() * rhs.data[2][1].clone()
                        + self.data[1][3].clone() * rhs.data[3][1].clone(),
                    self.data[1][0].clone() * rhs.data[0][2].clone()
                        + self.data[1][1].clone() * rhs.data[1][2].clone()
                        + self.data[1][2].clone() * rhs.data[2][2].clone()
                        + self.data[1][3].clone() * rhs.data[3][2].clone(),
                    self.data[1][0].clone() * rhs.data[0][3].clone()
                        + self.data[1][1].clone() * rhs.data[1][3].clone()
                        + self.data[1][2].clone() * rhs.data[2][3].clone()
                        + self.data[1][3].clone() * rhs.data[3][3].clone(),
                ],
                [
                    self.data[2][0].clone() * rhs.data[0][0].clone()
                        + self.data[2][1].clone() * rhs.data[1][0].clone()
                        + self.data[2][2].clone() * rhs.data[2][0].clone()
                        + self.data[2][3].clone() * rhs.data[3][0].clone(),
                    self.data[2][0].clone() * rhs.data[0][1].clone()
                        + self.data[2][1].clone() * rhs.data[1][1].clone()
                        + self.data[2][2].clone() * rhs.data[2][1].clone()
                        + self.data[2][3].clone() * rhs.data[3][1].clone(),
                    self.data[2][0].clone() * rhs.data[0][2].clone()
                        + self.data[2][1].clone() * rhs.data[1][2].clone()
                        + self.data[2][2].clone() * rhs.data[2][2].clone()
                        + self.data[2][3].clone() * rhs.data[3][2].clone(),
                    self.data[2][0].clone() * rhs.data[0][3].clone()
                        + self.data[2][1].clone() * rhs.data[1][3].clone()
                        + self.data[2][2].clone() * rhs.data[2][3].clone()
                        + self.data[2][3].clone() * rhs.data[3][3].clone(),
                ],
                [
                    self.data[3][0].clone() * rhs.data[0][0].clone()
                        + self.data[3][1].clone() * rhs.data[1][0].clone()
                        + self.data[3][2].clone() * rhs.data[2][0].clone()
                        + self.data[3][3].clone() * rhs.data[3][0].clone(),
                    self.data[3][0].clone() * rhs.data[0][1].clone()
                        + self.data[3][1].clone() * rhs.data[1][1].clone()
                        + self.data[3][2].clone() * rhs.data[2][1].clone()
                        + self.data[3][3].clone() * rhs.data[3][1].clone(),
                    self.data[3][0].clone() * rhs.data[0][2].clone()
                        + self.data[3][1].clone() * rhs.data[1][2].clone()
                        + self.data[3][2].clone() * rhs.data[2][2].clone()
                        + self.data[3][3].clone() * rhs.data[3][2].clone(),
                    self.data[3][0].clone() * rhs.data[0][3].clone()
                        + self.data[3][1].clone() * rhs.data[1][3].clone()
                        + self.data[3][2].clone() * rhs.data[2][3].clone()
                        + self.data[3][3].clone() * rhs.data[3][3].clone(),
                ],
            ],
        }
    }
}

/// Pack of operators definition module
impl<
        T: Copy
            + Neg<Output = T>
            + Sub<T, Output = T>
            + Add<T, Output = T>
            + Mul<T, Output = T>
            + Div<T, Output = T>,
    > Mat4x4<T>
{
    /// Determinant getting function
    /// * Returns determinant of this matrix
    pub fn determinant(&self) -> T {
        self.data[0][0]
            * (self.data[1][1] * self.data[2][2] * self.data[3][3]
                + self.data[1][2] * self.data[2][3] * self.data[3][1]
                + self.data[1][3] * self.data[2][1] * self.data[3][2]
                - self.data[1][1] * self.data[2][3] * self.data[3][2]
                - self.data[1][2] * self.data[2][1] * self.data[3][3]
                - self.data[1][3] * self.data[2][2] * self.data[3][1])
            - self.data[0][1]
                * (self.data[0][1] * self.data[2][2] * self.data[3][3]
                    + self.data[0][2] * self.data[2][3] * self.data[3][1]
                    + self.data[0][3] * self.data[2][1] * self.data[3][2]
                    - self.data[0][1] * self.data[2][3] * self.data[3][2]
                    - self.data[0][2] * self.data[2][1] * self.data[3][3]
                    - self.data[0][3] * self.data[2][2] * self.data[3][1])
            + self.data[0][2]
                * (self.data[0][1] * self.data[1][2] * self.data[3][3]
                    + self.data[0][2] * self.data[1][3] * self.data[3][1]
                    + self.data[0][3] * self.data[1][1] * self.data[3][2]
                    - self.data[0][1] * self.data[1][3] * self.data[3][2]
                    - self.data[0][2] * self.data[1][1] * self.data[3][3]
                    - self.data[0][3] * self.data[1][2] * self.data[3][1])
            - self.data[0][3]
                * (self.data[0][1] * self.data[1][2] * self.data[2][3]
                    + self.data[0][2] * self.data[1][3] * self.data[2][1]
                    + self.data[0][3] * self.data[1][1] * self.data[2][2]
                    - self.data[0][1] * self.data[1][3] * self.data[2][2]
                    - self.data[0][2] * self.data[1][1] * self.data[2][3]
                    - self.data[0][3] * self.data[1][2] * self.data[2][1])
    } // fn determinant

    /// Matrix inversion getting function
    /// * Returns this matrix inersed
    pub fn inversed(&self) -> Self {
        let determ_00 = self.data[1][1] * self.data[2][2] * self.data[3][3]
            + self.data[1][2] * self.data[2][3] * self.data[3][1]
            + self.data[1][3] * self.data[2][1] * self.data[3][2]
            - self.data[1][1] * self.data[2][3] * self.data[3][2]
            - self.data[1][2] * self.data[2][1] * self.data[3][3]
            - self.data[1][3] * self.data[2][2] * self.data[3][1];
        let determ_01 = self.data[0][1] * self.data[2][2] * self.data[3][3]
            + self.data[0][2] * self.data[2][3] * self.data[3][1]
            + self.data[0][3] * self.data[2][1] * self.data[3][2]
            - self.data[0][1] * self.data[2][3] * self.data[3][2]
            - self.data[0][2] * self.data[2][1] * self.data[3][3]
            - self.data[0][3] * self.data[2][2] * self.data[3][1];
        let determ_02 = self.data[0][1] * self.data[1][2] * self.data[3][3]
            + self.data[0][2] * self.data[1][3] * self.data[3][1]
            + self.data[0][3] * self.data[1][1] * self.data[3][2]
            - self.data[0][1] * self.data[1][3] * self.data[3][2]
            - self.data[0][2] * self.data[1][1] * self.data[3][3]
            - self.data[0][3] * self.data[1][2] * self.data[3][1];
        let determ_03 = self.data[0][1] * self.data[1][2] * self.data[2][3]
            + self.data[0][2] * self.data[1][3] * self.data[2][1]
            + self.data[0][3] * self.data[1][1] * self.data[2][2]
            - self.data[0][1] * self.data[1][3] * self.data[2][2]
            - self.data[0][2] * self.data[1][1] * self.data[2][3]
            - self.data[0][3] * self.data[1][2] * self.data[2][1];

        let determ = self.data[0][0] * determ_00 - self.data[0][1] * determ_01
            + self.data[0][2] * determ_02
            - self.data[0][3] * determ_03;

        Self {
            data: [
                [
                    self.data[0][0] * determ_00 / determ,
                    -self.data[0][1] * determ_01 / determ,
                    self.data[0][2] * determ_02 / determ,
                    -self.data[0][3] * determ_03 / determ,
                ],
                [
                    -self.data[1][0]
                        * (self.data[0][1] * self.data[2][2] * self.data[3][3]
                            + self.data[0][2] * self.data[2][3] * self.data[3][1]
                            + self.data[0][3] * self.data[2][1] * self.data[3][2]
                            - self.data[0][1] * self.data[2][3] * self.data[3][2]
                            - self.data[0][2] * self.data[2][1] * self.data[3][3]
                            - self.data[0][3] * self.data[2][2] * self.data[3][1])
                        / determ,
                    self.data[1][1]
                        * (self.data[0][0] * self.data[2][2] * self.data[3][3]
                            + self.data[0][2] * self.data[2][3] * self.data[3][0]
                            + self.data[0][3] * self.data[2][0] * self.data[3][2]
                            - self.data[0][0] * self.data[2][3] * self.data[3][2]
                            - self.data[0][2] * self.data[2][0] * self.data[3][3]
                            - self.data[0][3] * self.data[2][2] * self.data[3][0])
                        / determ,
                    -self.data[1][2]
                        * (self.data[0][0] * self.data[2][1] * self.data[3][3]
                            + self.data[0][1] * self.data[2][3] * self.data[3][0]
                            + self.data[0][3] * self.data[2][0] * self.data[3][1]
                            - self.data[0][0] * self.data[2][3] * self.data[3][1]
                            - self.data[0][1] * self.data[2][0] * self.data[3][3]
                            - self.data[0][3] * self.data[2][1] * self.data[3][0])
                        / determ,
                    self.data[1][3]
                        * (self.data[0][0] * self.data[2][1] * self.data[3][2]
                            + self.data[0][1] * self.data[2][2] * self.data[3][0]
                            + self.data[0][2] * self.data[2][0] * self.data[3][1]
                            - self.data[0][0] * self.data[2][2] * self.data[3][1]
                            - self.data[0][1] * self.data[2][0] * self.data[3][2]
                            - self.data[0][2] * self.data[2][1] * self.data[3][0])
                        / determ,
                ],
                [
                    self.data[2][0]
                        * (self.data[0][1] * self.data[1][2] * self.data[3][3]
                            + self.data[0][2] * self.data[1][3] * self.data[3][1]
                            + self.data[0][3] * self.data[1][1] * self.data[3][2]
                            - self.data[0][1] * self.data[1][3] * self.data[3][2]
                            - self.data[0][2] * self.data[1][1] * self.data[3][3]
                            - self.data[0][3] * self.data[1][2] * self.data[3][1])
                        / determ,
                    -self.data[2][1]
                        * (self.data[0][0] * self.data[1][2] * self.data[3][3]
                            + self.data[0][2] * self.data[1][3] * self.data[3][0]
                            + self.data[0][3] * self.data[1][0] * self.data[3][2]
                            - self.data[0][0] * self.data[1][3] * self.data[3][2]
                            - self.data[0][2] * self.data[1][0] * self.data[3][3]
                            - self.data[0][3] * self.data[1][2] * self.data[3][0])
                        / determ,
                    self.data[2][2]
                        * (self.data[0][0] * self.data[1][1] * self.data[3][3]
                            + self.data[0][1] * self.data[1][3] * self.data[3][0]
                            + self.data[0][3] * self.data[1][0] * self.data[3][1]
                            - self.data[0][0] * self.data[1][3] * self.data[3][1]
                            - self.data[0][1] * self.data[1][0] * self.data[3][3]
                            - self.data[0][3] * self.data[1][1] * self.data[3][0])
                        / determ,
                    -self.data[2][3]
                        * (self.data[0][0] * self.data[1][1] * self.data[3][2]
                            + self.data[0][1] * self.data[1][2] * self.data[3][0]
                            + self.data[0][2] * self.data[1][0] * self.data[3][1]
                            - self.data[0][0] * self.data[1][2] * self.data[3][1]
                            - self.data[0][1] * self.data[1][0] * self.data[3][2]
                            - self.data[0][2] * self.data[1][1] * self.data[3][0])
                        / determ,
                ],
                [
                    -self.data[3][0]
                        * (self.data[0][1] * self.data[1][2] * self.data[2][3]
                            + self.data[0][2] * self.data[1][3] * self.data[2][1]
                            + self.data[0][3] * self.data[1][1] * self.data[2][2]
                            - self.data[0][1] * self.data[1][3] * self.data[2][2]
                            - self.data[0][2] * self.data[1][1] * self.data[2][3]
                            - self.data[0][3] * self.data[1][2] * self.data[2][1])
                        / determ,
                    self.data[3][1]
                        * (self.data[0][0] * self.data[1][2] * self.data[2][3]
                            + self.data[0][2] * self.data[1][3] * self.data[2][0]
                            + self.data[0][3] * self.data[1][0] * self.data[2][2]
                            - self.data[0][0] * self.data[1][3] * self.data[2][2]
                            - self.data[0][2] * self.data[1][0] * self.data[2][3]
                            - self.data[0][3] * self.data[1][2] * self.data[2][0])
                        / determ,
                    -self.data[3][2]
                        * (self.data[0][0] * self.data[1][1] * self.data[2][3]
                            + self.data[0][1] * self.data[1][3] * self.data[2][0]
                            + self.data[0][3] * self.data[1][0] * self.data[2][1]
                            - self.data[0][0] * self.data[1][3] * self.data[2][1]
                            - self.data[0][1] * self.data[1][0] * self.data[2][3]
                            - self.data[0][3] * self.data[1][1] * self.data[2][0])
                        / determ,
                    self.data[3][3]
                        * (self.data[0][0] * self.data[1][1] * self.data[2][2]
                            + self.data[0][1] * self.data[1][2] * self.data[2][0]
                            + self.data[0][2] * self.data[1][0] * self.data[2][1]
                            - self.data[0][0] * self.data[1][2] * self.data[2][1]
                            - self.data[0][1] * self.data[1][0] * self.data[2][2]
                            - self.data[0][2] * self.data[1][1] * self.data[2][0])
                        / determ,
                ],
            ],
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
            ],
        }
    } // fn identity

    /// Rotation matrix getting function
    /// * `angle` - angle to create rotation matrix on
    /// * Returns rotation matrix
    pub fn rotate_x(angle: f32) -> Self {
        let sin = angle.sin();
        let cos = angle.cos();

        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos, sin, 0.0],
                [0.0, -sin, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    } // fn rotate_x

    /// Rotation matrix getting function
    /// * `angle` - angle to create rotation matrix on
    /// * Returns rotation matrix
    pub fn rotate_y(angle: f32) -> Self {
        let sin = angle.sin();
        let cos = angle.cos();

        Self {
            data: [
                [cos, 0.0, -sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    } // fn rotate_y

    /// Rotation matrix getting function
    /// * `angle` - angle to create rotation matrix on
    /// * Returns rotation matrix
    pub fn rotate_z(angle: f32) -> Self {
        let sin = angle.sin();
        let cos = angle.cos();

        Self {
            data: [
                [cos, sin, 0.0, 0.0],
                [-sin, cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    } // fn rotate_z
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
                [2.0 / (r - l), 0.0, 0.0, 0.0],
                [0.0, 2.0 / (t - b), 0.0, 0.0],
                [0.0, 0.0, -2.0 / (f - n), 0.0],
                [
                    -(r + l) / (r - l),
                    -(t + b) / (t - b),
                    -(f + n) / (f - n),
                    1.0,
                ],
            ],
        }
    } // fn projection_ortho

    /// Frustum projection matrix create function
    /// * `l`, `r` - left and right boundaries
    /// * `b`, `t` - bottom and top
    /// * `n`, `f` - near and far
    /// * Returns projection matrix
    pub fn projection_frustum(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4x4<f32> {
        Self {
            data: [
                [2.0 * n / (r - l), 0.0, 0.0, 0.0],
                [0.0, 2.0 * n / (t - b), 0.0, 0.0],
                [
                    (r + l) / (r - l),
                    (t + b) / (t - b),
                    -(f + n) / (f - n),
                    -1.0,
                ],
                [0.0, 0.0, -2.0 * n * f / (f - n), 0.0],
            ],
        }
    } // fn projection_frustum

    /// View projection matrix create function
    /// `l`, `r` - left and
    pub fn view(loc: Vec3<f32>, at: Vec3<f32>, approx_up: Vec3<f32>) -> Mat4x4<f32> {
        let dir = (at - loc).normalized();
        let right = (dir % approx_up).normalized();
        let up = (right % dir).normalized();

        Self {
            data: [
                [right.x, up.x, -dir.x, 0.0],
                [right.y, up.y, -dir.y, 0.0],
                [right.z, up.z, -dir.z, 0.0],
                [-(loc ^ right), -(loc ^ up), loc ^ dir, 1.0],
            ],
        }
    } // fn view
} // impl Mat4x4<f32>

impl Mat4x4<f32> {
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
                [
                    axis.x * axis.x * (1.0 - cosa) + cosa,
                    axis.x * axis.y * (1.0 - cosa) - axis.z * sina,
                    axis.x * axis.z * (1.0 - cosa) + axis.y * sina,
                    0.0,
                ],
                [
                    axis.y * axis.x * (1.0 - cosa) + axis.z * sina,
                    axis.y * axis.y * (1.0 - cosa) + cosa,
                    axis.y * axis.z * (1.0 - cosa) - axis.x * sina,
                    0.0,
                ],
                [
                    axis.z * axis.x * (1.0 - cosa) - axis.y * sina,
                    axis.z * axis.y * (1.0 - cosa) + axis.x * sina,
                    axis.z * axis.z * (1.0 - cosa) + cosa,
                    0.0,
                ],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    } // fn rotate

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
            ],
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
            ],
        }
    } // fn translate

    pub fn transform_vector(&self, v: Vec3<f32>) -> Vec3<f32> {
        Vec3 {
            x: v.x * self.data[0][0] + v.y * self.data[1][0] + v.z * self.data[2][0],
            y: v.x * self.data[0][1] + v.y * self.data[1][1] + v.z * self.data[2][1],
            z: v.x * self.data[0][2] + v.y * self.data[1][2] + v.z * self.data[2][2],
        }
    } // fn transform_vector

    pub fn transform_point(&self, v: Vec3<f32>) -> Vec3<f32> {
        Vec3 {
            x: v.x * self.data[0][0]
                + v.y * self.data[1][0]
                + v.z * self.data[2][0]
                + self.data[3][0],
            y: v.x * self.data[0][1]
                + v.y * self.data[1][1]
                + v.z * self.data[2][1]
                + self.data[3][1],
            z: v.x * self.data[0][2]
                + v.y * self.data[1][2]
                + v.z * self.data[2][2]
                + self.data[3][2],
        }
    } // fn transform_point

    pub fn transform_4x4(&self, v: Vec3<f32>) -> Vec3<f32> {
        let w =
            v.x * self.data[0][3] + v.y * self.data[1][3] + v.z * self.data[2][3] + self.data[3][3];

        Vec3 {
            x: (v.x * self.data[0][0]
                + v.y * self.data[1][0]
                + v.z * self.data[2][0]
                + self.data[3][0])
                / w,
            y: (v.x * self.data[0][1]
                + v.y * self.data[1][1]
                + v.z * self.data[2][1]
                + self.data[3][1])
                / w,
            z: (v.x * self.data[0][2]
                + v.y * self.data[1][2]
                + v.z * self.data[2][2]
                + self.data[3][2])
                / w,
        }
    } // fn transform_4x4
} // impl Mat4x4
