fn compile_shader_module(module_type: gl::types::GLenum, source: &str) -> Option<u32> {
    unsafe {
        let shader = gl::CreateShader(module_type);
        
        if shader == 0 {
            return None;
        }

        let ptr: *const i8 = std::mem::transmute(source.as_ptr());
        let len = source.len() as i32;

        gl::ShaderSource(shader, 1, &ptr, &len); 
        gl::CompileShader(shader);

        let mut compilation_status: i32 = 0;

        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compilation_status);
        if compilation_status != (gl::TRUE as i32) {
            let mut buffer = std::vec::Vec::<u8>::new();
            buffer.resize(512, 0);
            let mut buffer_length: i32 = 0;
            gl::GetShaderInfoLog(shader, 512, &mut buffer_length, std::mem::transmute(buffer.as_mut_ptr()));

            let log = String::from_utf8_lossy(buffer.as_slice());
            println!("Shader compilation error: {log}");

            gl::DeleteShader(shader);
            return None;
        }

        return Some(shader);
    }
} /* compile_shader_module */

pub fn compile(vert_source: Option<&str>, geom_source: Option<&str>, frag_source: Option<&str>) -> Option<u32> {
    unsafe {
        let mut build_descriptions: [(u32, Option<&str>, Option<u32>); 3] = [
            (gl::VERTEX_SHADER,   vert_source, None),
            (gl::FRAGMENT_SHADER, frag_source, None),
            (gl::GEOMETRY_SHADER, geom_source, None),
        ];

        for descr in &mut build_descriptions {
            match descr.1 {
                Some(source) => descr.2 = compile_shader_module(descr.0, source),
                None => continue,
            }
        }

        let mut program = gl::CreateProgram();

        if program != 0 {
            for descr in &build_descriptions {
                match descr.2 {
                    Some(shader) => gl::AttachShader(program, shader),
                    None => {},
                }
            }

            gl::LinkProgram(program);

            let mut status: i32 = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            if status != (gl::TRUE as i32) {
                let mut buffer = std::vec::Vec::<u8>::new();
                buffer.resize(512, 0);
                let mut buffer_length: i32 = 0;
                gl::GetProgramInfoLog(program, 512, &mut buffer_length, std::mem::transmute(buffer.as_mut_ptr()));

                let log = String::from_utf8_lossy(buffer.as_slice());
                println!("Program linking error: {log}");
                gl::DeleteProgram(program);
                program = 0;
            }
        }

        for descr in &mut build_descriptions {
            match descr.2 {
                Some(shader) => gl::DeleteShader(shader),
                None => {}
            }
        }

        if program == 0 { None } else { Some(program) }
    }
} /* compile_shader_program */
