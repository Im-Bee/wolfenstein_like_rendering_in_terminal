#[derive(Copy, Clone)]
struct Vec2<T> {
    x: T,
    y: T,
}

impl core::fmt::Display for Vec2<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

fn points_distance(pos1: Vec2<f32>, pos2: Vec2<f32>) -> f32 {
    ((pos2.x - pos1.x).powf(2.) + (pos2.y - pos1.y).powf(2.)).sqrt()
}

#[cfg(windows)]
mod windows_errors {
    pub fn get_last_error() -> u32 {
        use winapi::um::errhandlingapi::GetLastError;
        
        unsafe { 
            GetLastError() 
        }
    }
}

mod terminal {
    pub mod output {
        use crate::Vec2;
        use core::ptr::null_mut;
        use std::{mem::swap, usize};

        pub const CHAR_EMPTY: u8 = ' ' as u8;
        pub const BLACK_BOX_CHAR: u8 = 178;
        pub const STRIP_BOX_CHAR: u8 = '-' as u8;
        pub const AT_CHAR: u8 = '@' as u8;
        pub const DASH_CHAR: u8 = '-' as u8;

        type Screen = Vec<u8>;

        const FRONT_INDEX: usize = 0;
        const BACK_INDEX: usize = 1;

        pub struct Renderer {
            screen_dimensions: Vec2<i16>,
            swap_chain: Vec<Screen>,
        }

        impl Renderer {
            pub fn new() -> Renderer {
                let mut r = Renderer { 
                    screen_dimensions: (Vec2 { x: (-1), y: (-1) }),
                    swap_chain: (Vec::new())
                };

                r.swap_chain.push(Screen::new());
                r.swap_chain.push(Screen::new());

                return r;
            }

            pub fn draw_point_unnormalized(
                &mut self,
                pos: Vec2<i32>,
                ch: u8) {

                if !self.check_if_in_boundries(pos) {
                    return;
                }

                self.swap_chain[BACK_INDEX]
                    [(self.screen_dimensions.x as i32 * pos.y + pos.x) as usize] = ch;
            }

            pub fn draw_point(
                &mut self,
                mut pos: Vec2<i32>,
                ch: u8) {

                // Normialize
                pos.y /= 2;

                self.draw_point_unnormalized(pos, ch);
            }

            pub fn draw_dot(
                &mut self,
                mut pos: Vec2<f32>,
                ch: u8) {

                self.draw_line(
                    Vec2 { x: (pos.x + 3.), y: (pos.y) },
                    Vec2 { x: (pos.x - 3.), y: (pos.y) },
                    ch);

                self.draw_line(
                    Vec2 { x: (pos.x), y: (pos.y + 3.) },
                    Vec2 { x: (pos.x), y: (pos.y - 3.) },
                    ch);
            }

            pub fn draw_line(
                &mut self,
                mut pos0: Vec2<f32>,
                mut pos1: Vec2<f32>,
                ch: u8) {

                let mut steep = false;

                if (pos0.x - pos1.x).abs() < (pos0.y - pos1.y).abs() {
                    swap(&mut pos0.x, &mut pos0.y);
                    swap(&mut pos1.x, &mut pos1.y);
                    steep = true
                }

                if pos0.x > pos1.x {
                    swap(&mut pos0.x, &mut pos1.x);
                    swap(&mut pos0.y, &mut pos1.y);
                }

                let dx = pos1.x - pos0.x;
                let dy = pos1.y - pos0.y;
                let derror: f32 = (dy / dx).abs();
                let mut error: f32 = 0.0;
                let mut y = pos0.y as i32;

                for x in pos0.x as i32..pos1.x as i32 {
                    if steep {
                        self.draw_point(Vec2 { x: (y), y: (x) }, ch);
                    }
                    else {
                        self.draw_point(Vec2 { x: (x), y: (y) }, ch);
                    }

                    error += derror;
                    if error > 0.5 {
                        if pos1.y > pos0.y {
                            y += 1;
                        }
                        else {
                            y -= 1;
                        }

                        error -= 1.0;
                    }
                }
            }

            pub fn update(&mut self) {
                self.resize();
                self.clear_whole_screen();
                // TODO: self.update_objs();
            }

            pub fn render(&mut self) {
                self.swap_screens();
                self.render_frame();
            }

            pub fn get_screen_dim(&self) -> &Vec2<i16> {
                &self.screen_dimensions
            }

            #[inline]
            fn get_front_screen(&mut self) -> &mut Screen {
                &mut self.swap_chain[FRONT_INDEX]
            }

            #[inline]
            fn get_back_screen(&mut self) -> &mut Screen {
                &mut self.swap_chain[BACK_INDEX]
            }

            #[inline]
            fn check_if_in_boundries(&self, pos: Vec2<i32>) -> bool {
                if (pos.x >= self.screen_dimensions.x as i32) ||
                    (pos.y >= self.screen_dimensions.y as i32) ||
                    (pos.x < 0) || (pos.y < 0) {
                        return false;
                }
                else {
                    return true;
                }
            }

            fn resize(&mut self) {
                self.screen_dimensions = get_dimensions();
                let len = self.screen_dimensions.x as usize * self.screen_dimensions.y as usize;

                if len != self.get_front_screen().len() || 
                    len != self.get_back_screen().len() {
                        self.get_back_screen().resize(len, CHAR_EMPTY);
                        self.get_front_screen().resize(len, CHAR_EMPTY);

                        self.force_paint_whole_screen();
                        self.swap_screens();
                        self.clear_whole_screen();
                }
            }

            #[inline]
            fn clear_whole_screen(&mut self) {
                for i in self.get_back_screen().iter_mut() {
                    *i = CHAR_EMPTY;
                }
            }

            #[inline]
            fn force_paint_whole_screen(&mut self) {
                for i in self.get_back_screen().iter_mut() {
                    *i = 1;
                }
            }

            #[inline]
            fn blackout_whole_screen(&mut self) {
                for i in self.get_back_screen().iter_mut() {
                    *i = BLACK_BOX_CHAR;
                }
            }

            #[inline]
            fn swap_screens(&mut self) {
                self.swap_chain.swap(FRONT_INDEX, BACK_INDEX);
            }

            fn render_frame(&mut self) {
                const INVALID_ANCHOR: usize = usize::max_value();
                let d = &self.screen_dimensions;
                let mut anchor: usize = INVALID_ANCHOR;
    
                #[cfg(debug_assertions)]
                {
                    return;
                }

                set_cursor_position(Vec2 
                    { 
                        x: 0,
                        y: 0,
                    });

                for i in 0..self.swap_chain[FRONT_INDEX].len() {
                    if (anchor == INVALID_ANCHOR) && 
                        (self.swap_chain[FRONT_INDEX][i] != self.swap_chain[BACK_INDEX][i]) {
                            anchor = i;
                    }

                    if (anchor != INVALID_ANCHOR) &&
                        (self.swap_chain[FRONT_INDEX][i] == self.swap_chain[BACK_INDEX][i]) {
                            set_cursor_position(Vec2 
                                { 
                                    x: anchor as i16 % d.x,  
                                    y: anchor as i16 / d.x,
                                });

                            output_array(
                                &self.swap_chain[FRONT_INDEX][anchor],
                                (i - anchor) as i16);

                            set_cursor_position(Vec2 
                                { 
                                    x: 0,
                                    y: 0,
                                });

                            anchor = INVALID_ANCHOR;
                    }
                }

                if anchor != INVALID_ANCHOR {
                    output_array(
                        &self.swap_chain[FRONT_INDEX][anchor],
                        (self.swap_chain[FRONT_INDEX].len() - 1 - anchor) as i16);
                }

                set_cursor_position(Vec2 
                { 
                    x: 0,
                    y: 0,
                });
            }
        }
        
        #[cfg(windows)]
        pub fn get_dimensions() -> Vec2<i16> {
            use winapi::um::processenv::GetStdHandle;
            use winapi::um::wincon::GetConsoleScreenBufferInfo;
            use winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO;
            use winapi::um::wincon::SMALL_RECT;
            use winapi::um::wincon::COORD;

            let mut csbi = CONSOLE_SCREEN_BUFFER_INFO {
                dwSize: COORD { X: (-1), Y: (-1) },
                dwCursorPosition: COORD { X: (-1), Y: (-1) },
                wAttributes: -1_i16 as u16,
                srWindow: SMALL_RECT { 
                    Left: (-1), 
                    Top: (-1), 
                    Right: (-1), 
                    Bottom: (-1) },
                    dwMaximumWindowSize: COORD { X: (-1), Y: (-1) },
            };
    
            #[cfg(debug_assertions)]
            {
                return Vec2 { x: 10, y: 10 };
            }

            unsafe { 
                if GetConsoleScreenBufferInfo(
                    GetStdHandle(STD_OUTPUT),
                    &mut csbi) == 0 {
                    panic!("Cannot get console info in winapi,\
                        GetLastError() returned {err_code}", 
                        err_code = crate::windows_errors::get_last_error());
                }

            }

            Vec2 { x: csbi.dwSize.X, y: csbi.dwSize.Y }
        }

        #[cfg(windows)]
        const STD_OUTPUT: u32 = -11_i32 as u32;

        #[cfg(windows)]
        fn set_cursor_position(dim: Vec2<i16>) {
            use winapi::um::processenv::GetStdHandle; 
            use winapi::um::wincon::SetConsoleCursorPosition;
            use winapi::um::wincon::COORD;

            unsafe { 
                if SetConsoleCursorPosition(
                    GetStdHandle(STD_OUTPUT),
                    COORD { X: (dim.x), Y: (dim.y) }) == 0 {
                    panic!("Cannot set cursor positon in winapi,\
                        GetLastError() returned {err_code}", 
                        err_code = crate::windows_errors::get_last_error());
                }

            }
        }

        #[cfg(windows)]
        fn output_array(arr_ptr: *const u8, arr_size: i16) {
            use winapi::ctypes::c_void;
            use winapi::um::consoleapi::WriteConsoleA;
            use winapi::um::processenv::GetStdHandle;

            unsafe {
                if WriteConsoleA(
                    GetStdHandle(STD_OUTPUT), 
                    arr_ptr as *const c_void,
                    arr_size as u32,
                    null_mut(),
                    null_mut()) == 0 {
                    panic!("Cannot wirte to console in winapi,\
                        GetLastError() returned {err_code}",
                        err_code = crate::windows_errors::get_last_error());
                }
            }
        }
    }

    pub mod input {
        use std::sync::atomic::Ordering;
        use std::sync::Arc;
        use std::sync::atomic;
        use std::ptr::null_mut;
        use std::thread::spawn;
        use winapi::shared::windef::HHOOK;

        #[cfg(windows)]
        pub mod keys {
            pub type KEY = u32;

            pub const KEY_X: KEY = 88;
            pub const KEY_E: KEY = 69;
            pub const KEY_Q: KEY = 81;
            pub const KEY_W: KEY = 87;
            pub const KEY_S: KEY = 83;
            pub const KEY_A: KEY = 65;
            pub const KEY_D: KEY = 68;
            pub const KEY_UP: KEY = 0;
        }

        pub struct Hook {
            key: Arc<atomic::AtomicU32>,
            thread_switch: Arc<atomic::AtomicBool>,
        }

        impl Hook {
            pub fn new() -> Hook {
                let mut r = Hook {
                    key: (Arc::new(atomic::AtomicU32::new((keys::KEY_UP).into()))),
                    thread_switch: Arc::new(atomic::AtomicBool::new(true.into())),
                };

                r.create_input_thread();
                return r;
            }

            pub fn end(&mut self) {
                self.thread_switch.store(false, Ordering::Relaxed);
            }

            pub fn get_key(&self) -> keys::KEY {
                self.key.load(Ordering::Relaxed)
            }

            fn create_input_thread(&mut self) {
                use winapi::shared::windef::HWND;
                use winapi::shared::windef::POINT;
                use winapi::um::winuser::MSG;
                use winapi::um::winuser::PeekMessageA;
                use winapi::um::winuser::PM_REMOVE;
                use winapi::um::winuser::PM_QS_INPUT;

                let switch_clone = self.thread_switch.clone();
                let key_clone = self.key.clone();

                spawn(move || {                    
                    let mut msg = MSG {
                        hwnd: 0 as HWND,
                        message: 0 as u32,
                        wParam: 0 as usize,
                        lParam: 0 as isize,
                        time: 0,
                        pt: POINT { x: 0, y: 0 },
                    }; 

                    let hook_id = set_up_kb_hook();

                    loop {
                        unsafe {
                            if PeekMessageA(
                                &mut msg,
                                -1_i32 as HWND,
                                0,
                                0,
                                PM_REMOVE  | PM_QS_INPUT) == 0 {
                                key_clone.store(_KEY, Ordering::Relaxed);
                            }
                        }

                        if !switch_clone.load(Ordering::Relaxed) {
                            break;
                        }
                    }

                    end_kb_hook(hook_id);
                });
            }
        }

        impl Drop for Hook {
            fn drop(&mut self) {
                self.end();
                clean_up();
            }
        }

        pub fn clean_up() {
            // let mut f = String::new();
            // let _x = std::io::stdin().read_line(&mut f);
        }

        #[cfg(windows)]
        const WH_KEYBOARD_LL: i32 = 13;

        #[cfg(windows)]
        fn set_up_kb_hook() -> HHOOK {
            use winapi::um::winuser::SetWindowsHookExA;

            #[expect(unused_assignments)]
            let mut r: HHOOK = null_mut();

            unsafe {
                r = SetWindowsHookExA(
                    WH_KEYBOARD_LL, 
                    Some(windows_ll_hook), 
                    null_mut(), 
                    0);

                if r as i32 == 0 {
                    panic!("Couldn't create a hook in winapi, \
                        GetLastError() returned {err_code}", 
                        err_code = crate::windows_errors::get_last_error());
                }

            }

            return r;
        }

        #[cfg(windows)]
        static mut _KEY: keys::KEY = keys::KEY_UP;

        #[cfg(windows)]
        unsafe extern "system" fn windows_ll_hook(
            code: i32, 
            w_param: usize, 
            l_param: isize) -> isize {
            use winapi::um::winuser::CallNextHookEx;
            use winapi::um::winuser::KBDLLHOOKSTRUCT;
            use winapi::um::winuser::WM_KEYDOWN;
            use winapi::um::winuser::WM_KEYUP;

            let kbd: &KBDLLHOOKSTRUCT = (l_param as *const KBDLLHOOKSTRUCT).as_ref().unwrap();

            if w_param == WM_KEYDOWN as usize {
                _KEY = kbd.vkCode;
            }
            else if w_param == WM_KEYUP as usize {
                _KEY = keys::KEY_UP;
            }

            CallNextHookEx(null_mut(), code, w_param, l_param)
        }

        #[cfg(windows)]
        fn end_kb_hook(hk: HHOOK) {
            use winapi::um::winuser::UnhookWindowsHookEx;

            unsafe {
                if UnhookWindowsHookEx(hk) == 0 {
                    panic!("Couldn't unhook keyboard hook in winapi, \
                        GetLastError() returned {err_code}", 
                        err_code = crate::windows_errors::get_last_error());
                }
            }
        }
    }
}

mod game_logic {
    use std::usize;
    use std::f32::consts::PI;
    use std::time::{Duration, Instant};
    use crate::points_distance;
    use crate::terminal::output::{
        DASH_CHAR, 
        AT_CHAR, 
        BLACK_BOX_CHAR, 
        STRIP_BOX_CHAR};
    use crate::{
        terminal::{
            input::keys, output::Renderer},
        Vec2};

    const TICK_DURATION: Duration = Duration::from_millis(600);

    const PLAYER_ROTATION_SPEED: f32 = 0.1;

    const TWO_PI: f32 = 6.283185;
    const HALF_PI: f32 = 1.570795;
    const DEGREE: f32 = 57.29578;
    const RADIAN: f32 = 0.01745329;

    pub enum ViewMode {
        Mode2d,
        Mode3d,
        Mode2dAnd3d,
    }

    pub struct Game {
        ticks: Instant,
        current_map: Map,
        main_player: MainPlayer,
        camera: Camera,
    }

    struct Actor {
        position: Vec2<f32>,
        pitch: f32,
        movement_speed: f32,
    }

    struct MainPlayer {
        actor: Actor,
    }

    struct Map {
        topography: Vec<i32>,
        sqare_width: f32,
        topography_y: i32,
        topography_x: i32,
    }

    struct Camera {
        max_visible_distance: i32,
        fov: f32,
    }

    impl Game {
        pub fn new() -> Game {
            let new_main_player = MainPlayer {
                actor: Actor {
                    position: Vec2 { x: 50., y: 70. },
                    pitch: 11.44 * RADIAN,
                    movement_speed: 2.5,
                }
            };
            
            let new_map = Map {
                topography: 
                    [
                      1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                      1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                      1, 0, 0, 1, 1, 0, 0, 0, 0, 1,
                      1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                      1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 
                      1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 
                      1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 
                      1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 
                      1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 
                      1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 
                    ]
                    .to_vec(),
                topography_y: 10,
                topography_x: 10,
                sqare_width: 25.,
            };

            let new_camera = Camera {
                max_visible_distance: 15,
                fov: 90.,
            };

            Game {
                ticks: Instant::now(),
                current_map: new_map,
                main_player: new_main_player,
                camera: new_camera,
            }
        }

        pub fn update(
            &mut self,
            output: &mut Renderer,
            input: keys::KEY,
            mode: ViewMode) {

            let t = Instant::now();
            let dt = (t - self.ticks).as_millis() / TICK_DURATION.as_millis();
            if dt >= 1 {
                self.ticks = t;
            }
            
            if input != keys::KEY_UP {

                let mut top_left = self.main_player.actor.position;
                let mut pitch = self.main_player.actor.pitch;

                top_left.x -= self.main_player.actor.movement_speed / 2.;
                top_left.y -= self.main_player.actor.movement_speed / 2.;

                if input == keys::KEY_W {
                    let hit = cast_ray(
                        &self.main_player.actor.position, 
                        &pitch,
                        &top_left, 
                        &self.main_player.actor.movement_speed, 
                        &self.main_player.actor.movement_speed);

                    self.main_player.actor.position = hit.0;
                }
                    
                if input == keys::KEY_D {
                    pitch = normalize_angle(pitch + HALF_PI);

                    let hit = cast_ray(
                        &self.main_player.actor.position, 
                        &pitch,
                        &top_left, 
                        &self.main_player.actor.movement_speed, 
                        &self.main_player.actor.movement_speed);

                    self.main_player.actor.position = hit.0;
                }

                if input == keys::KEY_S {
                    pitch = normalize_angle(pitch + PI);

                    let hit = cast_ray(
                        &self.main_player.actor.position, 
                        &pitch,
                        &top_left, 
                        &self.main_player.actor.movement_speed, 
                        &self.main_player.actor.movement_speed);

                    self.main_player.actor.position = hit.0;
                }

                if input == keys::KEY_A {
                    pitch = normalize_angle(pitch + PI + HALF_PI);

                    let hit = cast_ray(
                        &self.main_player.actor.position, 
                        &pitch,
                        &top_left, 
                        &self.main_player.actor.movement_speed, 
                        &self.main_player.actor.movement_speed);

                    self.main_player.actor.position = hit.0;
                }

                if input == keys::KEY_E {
                    self.main_player.actor.pitch += PLAYER_ROTATION_SPEED;
                }

                if input == keys::KEY_Q {
                    self.main_player.actor.pitch -= PLAYER_ROTATION_SPEED;
                }

                self.main_player.actor.pitch = normalize_angle(self.main_player.actor.pitch);
            }
            
            self.calculate_and_draw(output, &mode);

            println!(
                "PITCH: {:03.4} | COORD: [x: {:02.04}, y: {:02.04}]",
                self.main_player.actor.pitch,
                self.main_player.actor.position.x,
                self.main_player.actor.position.y);
        }

        fn calculate_and_draw(
            &mut self,
            output: &mut Renderer,
            mode: &ViewMode) {
        
            let mut current_ray_pos: Vec2::<f32>;
            let mut current_ray_pitch = self.main_player.actor.pitch - (self.camera.fov / 2. * RADIAN);
            
            // Preallocate variables for calculations
            let mut ray_line = 0.;
            let dx = output.get_screen_dim().x as f32 / self.camera.fov;
            let dy = output.get_screen_dim().y as f32 / (self.camera.max_visible_distance as f32 * self.current_map.sqare_width);
            let mut which_axis: Axis = Axis::OnX;
            let mut ray_distance: f32;

            for _ in 0..(self.camera.fov as i32) {
                current_ray_pos = self.main_player.actor.position;
                current_ray_pitch = normalize_angle(current_ray_pitch);

                for _ in 0..self.camera.max_visible_distance {
                    // Check in which square we are
                    let current_square = self.calculate_current_square(current_ray_pos);

                    let topography_index = (self.current_map.topography_x * current_square.y + current_square.x) as usize; 
                    if topography_index >= self.current_map.topography.len() || 
                        self.current_map.topography[topography_index] == 1 {
                            // Hit!
                            break;
                    }

                    let current_top_left_of_square = Vec2::<f32> {
                        x: current_square.x as f32 * self.current_map.sqare_width,
                        y: current_square.y as f32 * self.current_map.sqare_width,
                    };
                    
                    let hit = cast_ray(
                        &current_ray_pos,
                        &current_ray_pitch,
                        &current_top_left_of_square,
                        &self.current_map.sqare_width,
                        &self.current_map.sqare_width);

                    current_ray_pos = hit.0;
                    which_axis = hit.1;
                }
                
                ray_line += dx;
                current_ray_pitch += RADIAN;
            
                match mode {
                    ViewMode::Mode2d => {
                        match which_axis {
                            Axis::OnX => {
                                output.draw_line(
                                    self.main_player.actor.position,
                                    current_ray_pos,
                                    BLACK_BOX_CHAR);
                            }
                            Axis::OnY => {
                                output.draw_line(
                                    self.main_player.actor.position,
                                    current_ray_pos,
                                    STRIP_BOX_CHAR);
                            }
                        }
                        // output.draw_dot(y_res, BLACK_BOX_CHAR);
                    }

                    ViewMode::Mode3d => {
                        ray_distance = points_distance(self.main_player.actor.position, current_ray_pos).ceil();

                        // Hit the same ray for dx amount
                        for i in 0..(dx + 1.) as i32 {
                            let up = Vec2 { x: (ray_line + i as f32), y: (0. + (ray_distance * dy) * 1.2) };
                            let down = Vec2 { x: (ray_line + i as f32), y: (1.8 * (output.get_screen_dim().y as f32 - (ray_distance * dy))) };
    
                            if up.y > down.y {
                                break;
                            }

                            match which_axis {
                                Axis::OnX => {
                                    output.draw_line(
                                        up,
                                        down,
                                        BLACK_BOX_CHAR);
                                }

                                Axis::OnY => {
                                    output.draw_line(
                                        up,
                                        down,
                                        STRIP_BOX_CHAR);
                                }
                            }
                        }
                    }

                    ViewMode::Mode2dAnd3d => { 
                        ray_distance = points_distance(self.main_player.actor.position, current_ray_pos).ceil();

                        // Hit the same ray for dx amount
                        for i in 0..(dx + 1.) as i32 {
                            match which_axis {
                                Axis::OnX => {
                                    output.draw_line(
                                        Vec2 { x: (ray_line + i as f32), y: (0. + (ray_distance * dy)) },
                                        Vec2 { x: (ray_line + i as f32), y: (output.get_screen_dim().y as f32 * 1.15 - (ray_distance * dy)) },
                                        BLACK_BOX_CHAR);
                                    output.draw_line(
                                        self.main_player.actor.position,
                                        current_ray_pos,
                                        BLACK_BOX_CHAR);
                                }

                                Axis::OnY => {
                                    output.draw_line(
                                        Vec2 { x: (ray_line + i as f32), y: (0. + (ray_distance * dy)) },
                                        Vec2 { x: (ray_line + i as f32), y: (output.get_screen_dim().y as f32 * 1.15 - (ray_distance * dy)) },
                                        STRIP_BOX_CHAR);
                                    output.draw_line(
                                        self.main_player.actor.position,
                                        current_ray_pos,
                                        STRIP_BOX_CHAR);
                                }
                            }
                        }
                    }
                }
            }
        }

        #[inline]
        fn calculate_current_square(
            &mut self,
            pos: Vec2<f32>) -> Vec2<i32> {
            Vec2::<i32> {
                x: (pos.x / self.current_map.sqare_width).floor() as i32,
                y: (pos.y / self.current_map.sqare_width).floor() as i32,
            }
        }
    }

    enum Axis {
        OnX,
        OnY,
    }

    fn normalize_angle(mut angle: f32) -> f32 {
        while angle < 0. {
            angle += TWO_PI;
        }
        while angle > TWO_PI {
            angle -= TWO_PI;
        }
        angle
    }

    fn cast_ray(
        starting_pos: &Vec2<f32>,
        pitch: &f32,
        boundry_top_left: &Vec2<f32>,
        x_boundry: &f32,
        y_boundry: &f32) -> (Vec2<f32>, Axis) {

        let error = 0.05;

        // Preallocate variables
        let mut a: f32;
        let mut o: f32;
        let y_res: Vec2<f32>;
        let x_res: Vec2<f32>;
        let mut final_pos: Vec2<f32>;
        let final_axis: Axis;
        let hit_on_f_y: bool;
        let hit_on_f_x: bool;

        let current_relative_pos = Vec2::<f32> {
            x: starting_pos.x - boundry_top_left.x,
            y: starting_pos.y - boundry_top_left.y,
        };

        // Decide should we calculate top or bottom ray for the y axis

        // Its top
        if !(*pitch > HALF_PI && *pitch < PI + HALF_PI) {
            a = current_relative_pos.y;
            o = pitch.tan() * a;

            y_res = Vec2 {
                x: starting_pos.x + o,
                y: starting_pos.y - current_relative_pos.y,
            };

            hit_on_f_y = true;
        }
        // Its bottom
        else {
            a = y_boundry - current_relative_pos.y;
            o = (pitch + PI).tan() * a;

            y_res = Vec2 {
                x: starting_pos.x - o,
                y: starting_pos.y - current_relative_pos.y + y_boundry,
            };

            hit_on_f_y = false;
        }

        // Decide should we calculate right or left ray for the x axis

        // Its right 
        if *pitch < PI && *pitch > 0. {
            a = x_boundry - current_relative_pos.x;
            o = (pitch - HALF_PI).tan() * a;

            x_res = Vec2 {
                x: starting_pos.x - current_relative_pos.x + x_boundry,
                y: starting_pos.y + o,
            };

            hit_on_f_x = true;
        }
        // Its left
        else {
            a = current_relative_pos.x;
            o = (pitch - PI - HALF_PI).tan() * a;

            x_res = Vec2 {
                x: starting_pos.x - current_relative_pos.x,
                y: starting_pos.y - o,
            };

            hit_on_f_x = false;
        }

        // Decide which result is correct and fits in boundries
        if y_res.x >= boundry_top_left.x &&
            y_res.x <= boundry_top_left.x + x_boundry {
                final_pos = y_res;
                final_axis = Axis::OnY;
        }
        else {
            final_pos = x_res;
            final_axis = Axis::OnX;
        } 

        // Jump over square border
        if hit_on_f_y { 
            final_pos.y -= error;
        }
        else {
            final_pos.y += error;
        }
        if hit_on_f_x { 
            final_pos.x += error;
        }
        else {
            final_pos.x -= error;
        }

        return (final_pos, final_axis);
    }
}

fn main() {
    use std::thread::sleep;
    use std::time::Duration;

    let input = terminal::input::Hook::new();
    let mut render = terminal::output::Renderer::new();
    let mut game = game_logic::Game::new();

    loop {
        sleep(Duration::from_millis(50));
        render.update();
        game.update(
            &mut render,
            input.get_key(),
            game_logic::ViewMode::Mode3d);

        render.render();

        if input.get_key() == terminal::input::keys::KEY_X {
            break;
        }
    }
}
