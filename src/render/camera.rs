/// WAT3RS Project
/// `File` render/camera.rs
/// `Description` Render camera implementation file
/// `Author` TioT2
/// `Last changed` 17.02.2024

pub use crate::math::*;


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
    pub size: Vec2f,

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
    extent: Vec2<usize>,
} // struct camera

impl Camera {
    /// Camera create function
    /// * Returns new camera
    pub fn new() -> Self {
        let mut cam = Self {
            location: Location {
                direction: Vec3f::new(0.0, 0.0, -1.0),
                right: Vec3f::new(1.0, 0.0, 0.0),
                up: Vec3f::new(0.0, 1.0, 0.0),

                location: Vec3f::new(0.0, 0.0, 1.0),
                at: Vec3f::new(0.0, 0.0, 0.0),
            },

            projection: Projection {
                size: Vec2f::new(1.0, 1.0),
                near: 1.0,
                far: 1.0,
            },

            matrices: Matrices {
                view: Mat4x4f::identity(),
                projection: Mat4x4f::identity(),
                view_projection: Mat4x4f::identity(),
            },

            extent: Vec2::<usize>::new(0, 0),
        };

        cam.resize(Vec2::<usize>::new(800, 600));
        cam.set_projection(0.05, 1024.0, Vec2f::new(0.1, 0.1));

        cam
    }

    /// Camera setting function
    /// * `location` - new camera location
    /// * `at` - location camera points at
    /// * `approx_up` - approximate up direction
    pub fn set(&mut self, location: &Vec3f, at: &Vec3f, approx_up: &Vec3f) {
        let view = Mat4x4::view(location, at, approx_up);

        self.location.right     = Vec3f::new( view.data[0][0],  view.data[1][0],  view.data[2][0]);
        self.location.up        = Vec3f::new( view.data[0][1],  view.data[1][1],  view.data[2][1]);
        self.location.direction = Vec3f::new(-view.data[0][2], -view.data[1][2], -view.data[2][2]);

        self.location.location = *location;
        self.location.at = *at;

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
    pub fn set_projection(&mut self, near: f32, far: f32, size: Vec2f) {
        self.projection.near = near;
        self.projection.far = far;
        self.projection.size = size;

        let proj_ext = self.projection.size * if self.extent.x > self.extent.y {
            Vec2f::new(self.extent.x as f32 / self.extent.y as f32, 1.0)
        } else {
            Vec2f::new(1.0, self.extent.y as f32 / self.extent.x as f32)
        };

        self.matrices.projection = Mat4x4f::projection_frustum(-proj_ext.x / 2.0, proj_ext.x / 2.0, -proj_ext.y / 2.0, proj_ext.y / 2.0, self.projection.near, self.projection.far);
        self.matrices.view_projection = self.matrices.view * self.matrices.projection;
    } // fn set_projection

    /// Camera fitting for new resolution
    /// * `new_extent` - new resolution
    pub fn resize(&mut self, new_extent: Vec2<usize>) {
        if self.extent.x == new_extent.x && self.extent.y == new_extent.y {
            return;
        }
        self.extent = new_extent;

        let proj_ext = self.projection.size * if self.extent.x > self.extent.y {
            Vec2f::new(self.extent.x as f32 / self.extent.y as f32, 1.0)
        } else {
            Vec2f::new(1.0, self.extent.y as f32 / self.extent.x as f32)
        };

        self.matrices.projection = Mat4x4f::projection_frustum(-proj_ext.x / 2.0, proj_ext.x / 2.0, -proj_ext.y / 2.0, proj_ext.y / 2.0, self.projection.near, self.projection.far);
        self.matrices.view_projection = self.matrices.view * self.matrices.projection;
    } // fn resize
} // impl Camera

// file camera.rs
