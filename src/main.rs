// Shader for all entities exist in game
// uniforms: point_size: vec2, projection_size: vec2, point_color: vec3
mod entity_shader {
    pub const FRAG: &'static str = r#"
    #version 330 core

    layout(location = 0) in vec2 position;

    uniform vec3 point_color;
    uniform vec2 projection_size;

    out vec3 color;

    void main(void) {
        color = point_color;
        gl_Position = vec4(position / projection_size, 0, 1);
    }
    "#;

    pub const GEOM: &'static str = r#"
    #version 330 core

    layout(points) in;
    layout(triangle_strip, max_vertices = 4) out;

    uniform vec2 point_size;
    uniform vec2 projection_size;
    in vec3 color[];

    out vec3 gs_color;

    void main(void) {
        vec2 size = point_size / projection_size / 2;
        gl_Position = gl_in[0].gl_Position; gl_Position.xy += size * vec2(-1, -1); gs_color = color[0]; EmitVertex();
        gl_Position = gl_in[0].gl_Position; gl_Position.xy += size * vec2(-1, +1); gs_color = color[0]; EmitVertex();
        gl_Position = gl_in[0].gl_Position; gl_Position.xy += size * vec2(+1, -1); gs_color = color[0]; EmitVertex();
        gl_Position = gl_in[0].gl_Position; gl_Position.xy += size * vec2(+1, +1); gs_color = color[0]; EmitVertex();
        EndPrimitive();
    }
    "#;

    pub const VERT: &'static str = r#"
    #version 330 core
    layout(location = 0) out vec4 out_color;

    in vec3 gs_color;

    void main(void) {
        out_color = vec4(gs_color, 0);
    }
    "#;
} /* mod entity_shader */

pub mod linmath;
pub mod game;
pub mod shader;
pub mod timer;

type Vec2 = linmath::Vec2<f32>;

fn main() {
    let sdl_instance = sdl2::init().unwrap();
    let sdl_video = sdl_instance.video().unwrap();
    let mut sdl_event_pump = sdl_instance.event_pump().unwrap();

    let gl_attr = sdl_video.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 2);
    gl_attr.set_context_flags()
        .debug()
        .forward_compatible()
        .set();

    let window = sdl_video.window("pixel-mobs-guns", 800, 600)
        .opengl()
        .resizable()
        .build().expect("Error creating SDL2 window");

    assert_eq!(gl_attr.context_profile(), sdl2::video::GLProfile::Core);
    assert_eq!(gl_attr.context_version(), (3, 2));

    let gl_context = window.gl_create_context().expect("Error creating GL context");
    window.gl_make_current(&gl_context).expect("OpenGL context activation error");
    gl::load_with(|name| sdl_video.gl_get_proc_address(name) as *const _);

    let mut timer = timer::Timer::new(&sdl_instance);
    let mut engine = game::Engine::new();

    engine.enemies.push(game::Enemy {
        position: Vec2::new(0.2, 0.2),
        health: 0.0,
    });
    engine.enemies.push(game::Enemy {
        position: Vec2::new(-0.2, 0.2),
        health: 0.0,
    });
    engine.enemies.push(game::Enemy {
        position: Vec2::new(0.2, -0.2),
        health: 0.0,
    });
    engine.enemies.push(game::Enemy {
        position: Vec2::new(-0.2, -0.2),
        health: 0.0,
    });

    let shader = shader::compile(Some(entity_shader::FRAG), Some(entity_shader::GEOM), Some(entity_shader::VERT)).unwrap();
    let point_size_location: i32 = unsafe { gl::GetUniformLocation(shader, std::mem::transmute(b"point_size\0".as_ptr())) };
    let point_color_location: i32 = unsafe { gl::GetUniformLocation(shader, std::mem::transmute(b"point_color\0".as_ptr())) };
    let projection_size_location: i32 = unsafe { gl::GetUniformLocation(shader, std::mem::transmute(b"projection_size\0".as_ptr())) };

    let mut player_vertex_array: u32 = 0;
    let mut player_vertex_buffer: u32 = 0;

    let mut enemy_vertex_array: u32 = 0;
    let mut enemy_vertex_buffer: u32 = 0;

    let mut bullet_vertex_array: u32 = 0;
    let mut bullet_vertex_buffer: u32 = 0;

    let mut window_width = 800;
    let mut window_height = 600;

    let mut mouse_x: f32 = 0.0;
    let mut mouse_y: f32 = 0.0;

    // OpenGL resource generation
    unsafe {
        // Fill player data
        gl::GenBuffers(1, &mut player_vertex_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, player_vertex_buffer);
        let data: [f32; 2] = [0.5, 0.5];
        gl::BufferData(gl::ARRAY_BUFFER, 8, std::mem::transmute(data.as_ptr()), gl::STATIC_DRAW);

        gl::GenVertexArrays(1, &mut player_vertex_array);
        gl::BindVertexArray(player_vertex_array);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribFormat(0, 2, gl::FLOAT, gl::FALSE, 0);
        gl::BindVertexBuffer(0, player_vertex_buffer, 0, 8);

        // Generate bullet buffers
        gl::GenBuffers(1, &mut bullet_vertex_buffer);
        gl::GenVertexArrays(1, &mut bullet_vertex_array);

        gl::BindVertexArray(bullet_vertex_array);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribFormat(0, 2, gl::FLOAT, gl::FALSE, 0);
        gl::BindVertexBuffer(0, bullet_vertex_buffer, 0, 8);

        // Generate enemy buffers
        gl::GenBuffers(1, &mut enemy_vertex_buffer);
        gl::GenVertexArrays(1, &mut enemy_vertex_array);

        gl::BindVertexArray(enemy_vertex_array);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribFormat(0, 2, gl::FLOAT, gl::FALSE, 0);
        gl::BindVertexBuffer(0, enemy_vertex_buffer, 0, 8);
    }

    'main_loop: loop {
        let mut player_emit_bullet = false;

        'event_loop: loop {
            let event = match sdl_event_pump.poll_event() {
                Some(some_event) => some_event,
                None => break 'event_loop,
            };

            // switch on event
            match event {
                sdl2::event::Event::Window {win_event, ..} => {
                    match win_event {
                        sdl2::event::WindowEvent::SizeChanged(new_width, new_height) => {
                            unsafe {
                                window_width = new_width;
                                window_height = new_height;
                                gl::Viewport(0, 0, new_width, new_height);
                            }
                        }
                        _ => {}
                    }
                }
                sdl2::event::Event::KeyDown {keycode, ..} => {
                    match keycode {
                        Some(keycode_value) => match keycode_value {
                            sdl2::keyboard::Keycode::Space => player_emit_bullet = true,

                            _ => {}
                        }
                        None => {}
                    }
                }
                sdl2::event::Event::MouseMotion {x, y, ..} => {
                    mouse_x = x as f32 / window_width as f32;
                    mouse_y = y as f32 / window_height as f32;

                    mouse_x = mouse_x * 2.0 - 1.0;
                    mouse_y = 1.0 - mouse_y * 2.0;
                }
                sdl2::event::Event::Quit {..} => {
                    break 'main_loop;
                }
                _ => { }
            }
        }

        timer.update();

        let keyboard_state = sdl_event_pump.keyboard_state();
        let player_move_axis = Vec2::new(
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::L) as i32 as f32 - keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::J) as i32 as f32,
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::I) as i32 as f32 - keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::K) as i32 as f32,
        );
        engine.player.position += player_move_axis * 0.8 * timer.delta_time as f32;

        if player_emit_bullet {
            let velocity = (Vec2::new(mouse_x, mouse_y) - engine.player.position).normalized();
            let position = engine.player.position + velocity * 0.01;

            engine.bullets.push(game::Bullet{position, velocity});
        }

        engine.update(timer.delta_time as f32);

        let (projection_w, projection_h) = if window_width > window_height {
            (window_width as f32 / window_height as f32, 1.0)
        } else {
            (1.0, window_height as f32 / window_width as f32)
        };

        // response
        unsafe {
            // Update player
            gl::BindBuffer(gl::ARRAY_BUFFER, player_vertex_buffer);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, 8, std::mem::transmute([engine.player.position.x, engine.player.position.y].as_ptr()));

            // Update enemies
            let mut enemy_vertices = Vec::<f32>::with_capacity(engine.enemies.len() * 2);
            for enemy in &engine.enemies {
                enemy_vertices.push(enemy.position.x);
                enemy_vertices.push(enemy.position.y);
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, enemy_vertex_buffer);
            gl::BufferData(gl::ARRAY_BUFFER, enemy_vertices.len() as isize * 4, std::mem::transmute(enemy_vertices.as_ptr()), gl::STATIC_DRAW);

            // Update bullets
            let mut bullet_vertices = Vec::<f32>::with_capacity(engine.bullets.len() * 2);
            for bullet in &engine.bullets {
                bullet_vertices.push(bullet.position.x);
                bullet_vertices.push(bullet.position.y);
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, bullet_vertex_buffer);
            gl::BufferData(gl::ARRAY_BUFFER, bullet_vertices.len() as isize * 4, std::mem::transmute(bullet_vertices.as_ptr()), gl::STATIC_DRAW);

            // rendering
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.30, 0.47, 0.80, 1.00);

            gl::UseProgram(shader);
            gl::Uniform2f(projection_size_location, projection_w, projection_h);

            // Render bullets
            gl::Uniform2f(point_size_location, 0.02, 0.02);
            gl::Uniform3f(point_color_location, 1.0, 0.0, 0.0);
            gl::BindVertexArray(bullet_vertex_array);
            gl::DrawArrays(gl::POINTS, 0, engine.bullets.len() as i32);

            // Render player
            gl::Uniform2f(point_size_location, 0.05, 0.05);
            gl::Uniform3f(point_color_location, 0.0, 1.0, 0.0);
            gl::BindVertexArray(player_vertex_array);
            gl::DrawArrays(gl::POINTS, 0, 1);

            // Render enemies
            gl::Uniform2f(point_size_location, 0.10, 0.10);
            gl::Uniform3f(point_color_location, 0.55, 0.00, 1.00);
            gl::BindVertexArray(enemy_vertex_array);
            gl::DrawArrays(gl::POINTS, 0, engine.enemies.len() as i32);

            gl::Finish();
        }

        window.gl_swap_window();
    }

    // Clear all OpenGL-depentent staff
    unsafe {
        gl::DeleteVertexArrays(1, &enemy_vertex_array);
        gl::DeleteBuffers(1, &enemy_vertex_buffer);

        gl::DeleteVertexArrays(1, &bullet_vertex_array);
        gl::DeleteBuffers(1, &bullet_vertex_buffer);

        gl::DeleteVertexArrays(1, &player_vertex_array);
        gl::DeleteBuffers(1, &player_vertex_buffer);

        gl::DeleteProgram(shader);
    }

    println!("Hello, world!");
} /* main */
