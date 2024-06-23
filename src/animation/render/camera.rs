/// WAT3RS Project
/// `File` render/camera.rs
/// `Description` Render camera implementation file
/// `Author` TioT2
/// `Last changed` 17.02.2024

use crate::util::{math::Ext2, Ext2f, Mat4x4f, Vec2f, Vec3f};


/// Camera in space position info representation structure
#[derive(Copy, Clone)]
pub struct Location {
    /// Normalized vector from eye to point of view
    pub direction: Vec3f,

    /// Right direction
    pub right: Vec3f,

    /// Up direction
    pub up: Vec3f,

    /// Eye location
    pub location: Vec3f,

    /// Eye point of view
    pub at: Vec3f,
} // struct Location

/// Camera projection representation structure
#[derive(Copy, Clone)]
pub struct Projection {
    /// Projection size
    pub size: Ext2<f32>,

    /// Projection near plane
    pub near: f32,

    /// Projection far plane
    pub far: f32,
} // struct Projection

/// Camera projection matrix set representation structure
#[derive(Copy, Clone)]
pub struct Matrices {
    /// View matrix
    pub view: Mat4x4f,

    /// Projection matrix
    pub projection: Mat4x4f,

    /// View matrix with projection matrix product, actually cached value
    pub view_projection: Mat4x4f,
} // struct Matrices

/// Renderer camera representation structure
#[derive(Copy, Clone)]
pub struct Camera {
    location: Location,
    projection: Projection,
    matrices: Matrices,
    extent: Ext2<usize>,
} // struct camera

impl Default for Camera {
    fn default() -> Self {
        let mut cam = Self {
            location: Location {
                direction: Vec3f::new(0.0, 0.0, -1.0),
                right: Vec3f::new(1.0, 0.0, 0.0),
                up: Vec3f::new(0.0, 1.0, 0.0),

                location: Vec3f::new(0.0, 0.0, 1.0),
                at: Vec3f::new(0.0, 0.0, 0.0),
            },

            projection: Projection {
                size: Ext2f::new(1.0, 1.0),
                near: 1.0,
                far: 1.0,
            },

            matrices: Matrices {
                view: Mat4x4f::identity(),
                projection: Mat4x4f::identity(),
                view_projection: Mat4x4f::identity(),
            },

            extent: Ext2::new(0, 0),
        };

        cam.resize(Ext2::<usize>::new(800, 600));
        cam.set_projection(0.05, 4096.0, Ext2f::new(0.1, 0.1));

        cam
    }
}

impl Camera {
    /// Camera setting function
    /// * `location` - new camera location
    /// * `at` - location camera points at
    /// * `approx_up` - approximate up direction
    pub fn set(&mut self, location: Vec3f, at: Vec3f, approx_up: Vec3f) {
        let view = Mat4x4f::view(location, at, approx_up);

        self.location.right     = Vec3f::new( view.data[0][0],  view.data[1][0],  view.data[2][0]);
        self.location.up        = Vec3f::new( view.data[0][1],  view.data[1][1],  view.data[2][1]);
        self.location.direction = Vec3f::new(-view.data[0][2], -view.data[1][2], -view.data[2][2]);

        self.location.location = location;
        self.location.at = at;

        self.matrices.view = view;
        self.matrices.view_projection = self.matrices.view * self.matrices.projection;
    } // fn set

    /// Camera location getting function
    /// * Returns location info
    pub fn get_location(&self) -> &Location {
        &self.location
    } // fn get_location

    /// Camera projection getting function
    /// * Returns projection info
    pub fn get_projection(&self) -> &Projection {
        &self.projection
    } // fn get_projection

    /// Camera matrix getting function
    /// * Returns matrix info
    pub fn get_matrices(&self) -> &Matrices {
        &self.matrices
    } // fn get_matrices

    /// Camera projection setting function
    /// * `near` - projection plane distance
    /// * `far` - projection maximal distance
    /// * `size` - projection size
    pub fn set_projection(&mut self, near: f32, far: f32, size: Ext2f) {
        self.projection.near = near;
        self.projection.far = far;
        self.projection.size = size;

        let proj_ext: Ext2f = (if self.extent.w > self.extent.h {
            Vec2f::new(self.extent.w as f32 / self.extent.h as f32, 1.0)
        } else {
            Vec2f::new(1.0, self.extent.h as f32 / self.extent.w as f32)
        } * Into::<Vec2f>::into(self.projection.size.into_tuple())).into_tuple().into();

        self.matrices.projection = Mat4x4f::projection_frustum(-proj_ext.w / 2.0, proj_ext.w / 2.0, -proj_ext.h / 2.0, proj_ext.h / 2.0, self.projection.near, self.projection.far);
        self.matrices.view_projection = self.matrices.view * self.matrices.projection;
    } // fn set_projection

    /// Camera fitting for new resolution
    /// * `new_extent` - new resolution
    pub fn resize(&mut self, new_extent: Ext2<usize>) {
        if self.extent.w == new_extent.w && self.extent.h == new_extent.h {
            return;
        }
        self.extent = new_extent;

        let proj_ext: Ext2f = (if self.extent.w > self.extent.h {
            Vec2f::new(self.extent.w as f32 / self.extent.h as f32, 1.0)
        } else {
            Vec2f::new(1.0, self.extent.h as f32 / self.extent.w as f32)
        } * Into::<Vec2f>::into(self.projection.size.into_tuple())).into_tuple().into();

        self.matrices.projection = Mat4x4f::projection_frustum(-proj_ext.w / 2.0, proj_ext.w / 2.0, -proj_ext.h / 2.0, proj_ext.h / 2.0, self.projection.near, self.projection.far);
        self.matrices.view_projection = self.matrices.view * self.matrices.projection;
    } // fn resize
} // impl Camera

// file camera.rs
