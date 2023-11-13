pub struct Timer {
    sdl_timer: sdl2::TimerSubsystem,

    ticks_per_second: u64,
    initial_ticks: u64,
    last_ticks: u64,

    fps_counter: u32,
    fps_last_ticks: u64,

    pub fps: f64,
    pub time: f64,
    pub delta_time: f64,
}

impl Timer {
    pub fn new(sdl_instance: &sdl2::Sdl) -> Timer {
        let sdl_timer = sdl_instance.timer().unwrap();

        let ticks_per_second = sdl_timer.performance_frequency();
        let initial_ticks = sdl_timer.performance_counter();

        Timer {
            sdl_timer,
            ticks_per_second,
            initial_ticks,
            last_ticks: initial_ticks,
            fps_counter: 0,
            fps_last_ticks: initial_ticks,
            fps: 30.0,
            time: 0.0,
            delta_time: 0.001,
        }
    }

    pub fn update(&mut self) {
        let ticks = self.sdl_timer.performance_counter();

        self.time = (ticks - self.initial_ticks) as f64 / (self.ticks_per_second as f64);
        self.delta_time = (ticks - self.last_ticks) as f64 / self.ticks_per_second as f64;
        self.last_ticks = ticks;

        self.fps_counter += 1;
        if self.last_ticks - self.fps_last_ticks > self.ticks_per_second * 3 {
            self.fps = self.fps_counter as f64 / ((self.last_ticks - self.fps_last_ticks) as f64 / (self.ticks_per_second * 3) as f64);
            self.fps_counter = 0;
            self.fps_last_ticks = self.last_ticks;
        }
    }
}
