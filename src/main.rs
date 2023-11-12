// Shader for all entities exist in game
// uniforms: point_size: float, point_color: vec3
mod entity_shader {
    pub const FRAG: &'static str = r#"
    #version 330 core

    layout(location = 0) in vec2 position;

    void main(void) {
        gl_Position = vec4(position, 0, 1);
    }
    "#;

    pub const GEOM: &'static str = r#"
    #version 330 core

    layout(points) in;
    layout(triangle_strip, max_vertices = 4) out;

    uniform float point_size;

    void main(void) {
        gl_Position = gl_in[0].gl_Position + vec4(-point_size / 2, -point_size / 2, 0, 0); EmitVertex();
        gl_Position = gl_in[0].gl_Position + vec4(-point_size / 2, +point_size / 2, 0, 0); EmitVertex();
        gl_Position = gl_in[0].gl_Position + vec4(+point_size / 2, -point_size / 2, 0, 0); EmitVertex();
        gl_Position = gl_in[0].gl_Position + vec4(+point_size / 2, +point_size / 2, 0, 0); EmitVertex();
        EndPrimitive();
    }
    "#;

    pub const VERT: &'static str = r#"
    #version 330 core
    layout(location = 0) out vec4 out_color;

    uniform vec3 point_color;

    void main(void) {
        out_color = vec4(point_color, 0);
    }
    "#;
} /* mod entity_shader */

pub mod linmath;
pub mod game;
pub mod shader;

// type Vec2 = linmath::Vec2<f32>;
// type Vec3 = linmath::Vec3<f32>;

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

    // uniforms: float point_size, vec3 point_color
    let shader = shader::compile(Some(entity_shader::FRAG), Some(entity_shader::GEOM), Some(entity_shader::VERT)).unwrap();
    let point_size_location: i32 = unsafe { gl::GetUniformLocation(shader, std::mem::transmute(b"point_size\0".as_ptr())) };
    let point_color_location: i32 = unsafe { gl::GetUniformLocation(shader, std::mem::transmute(b"point_color\0".as_ptr())) };

    let mut player_vertex_array: u32 = 0;
    let mut player_vertex_buffer: u32 = 0;

    unsafe {
        gl::GenBuffers(1, &mut player_vertex_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, player_vertex_buffer);
        let data: [f32; 2] = [0.0, 0.0];
        gl::BufferData(gl::ARRAY_BUFFER, 8, std::mem::transmute(data.as_ptr()), gl::STATIC_DRAW);

        gl::GenVertexArrays(1, &mut player_vertex_array);
        gl::BindVertexArray(player_vertex_array);
        gl::EnableVertexAttribArray(0);
        gl::BindVertexBuffer(0, player_vertex_buffer, 0, 8);
    }

    'main_loop: loop {
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
                                gl::Viewport(0, 0, new_width, new_height);
                            }
                        },
                        _ => {}
                    }
                },
                sdl2::event::Event::Quit {..} => {
                    println!("Quit event.");
                    break 'main_loop;
                }
                _ => {},
            }
        }

        // response
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.30, 0.47, 0.80, 1.00);

            gl::BindVertexArray(player_vertex_array);

            gl::UseProgram(shader);
            gl::Uniform1f(point_size_location, 0.5);
            gl::Uniform3f(point_color_location, 0.0, 1.0, 0.0);

            gl::DrawArrays(gl::POINTS, 0, 1);

            gl::Finish();
        }

        window.gl_swap_window();
    }

    unsafe {
        gl::DeleteVertexArrays(1, &player_vertex_array);
        gl::DeleteBuffers(1, &player_vertex_buffer);
        gl::DeleteProgram(shader);
    }

    println!("Hello, world!");
} /* main */
