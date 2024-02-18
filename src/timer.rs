/// Timer representation structure
pub struct Timer {
    start_time_point: std::time::Instant,
    time_point: std::time::Instant,
    fps_time_point: std::time::Instant,
    time: f32,
    delta_time: f32,
    fps: f32,
    fps_counter: u32,
    fps_duration: f32,
}

impl Timer {
    /// Timer constructor.
    /// * Returns newly created timer with count, starting from creation moment
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            start_time_point: now.clone(),
            time_point: now.clone(),
            fps_time_point: now.clone(),
            time: 0.0,
            delta_time: 0.01,
            fps: 30.0,
            fps_counter: 0,
            fps_duration: 1.0,
        }
    }

    /// Timer resetting function.
    /// Sets timer starting point to 0.
    pub fn reset_time(&mut self) {
        let now = std::time::Instant::now();
        self.start_time_point = now.clone();
        self.time_point = now.clone();
        self.fps_time_point = now.clone();
        self.fps_counter = 0;

        self.time = 0.0;
    }

    /// Timer duration update function.
    pub fn response(&mut self) {
        let now = std::time::Instant::now();

        self.time = (now - self.start_time_point).as_secs_f32();
        self.delta_time = (now - self.time_point).as_secs_f32();

        self.fps_counter += 1;

        let fps_duration = (now - self.fps_time_point).as_secs_f32();
        if fps_duration >= self.fps_duration {
            self.fps = self.fps_counter as f32 / fps_duration;
            self.fps_time_point = now;
            self.fps_counter = 0;
        }

        self.time_point = now;
    }

    /// Time getting function
    pub fn get_time(&self) -> f32 {
        self.time
    }

    /// Time between neighbour updates getting function
    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }

    /// FPS getting function
    pub fn get_fps(&self) -> f32 {
        self.fps
    }
}
