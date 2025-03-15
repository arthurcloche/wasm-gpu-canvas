use wasm_bindgen::prelude::*;
use web_sys::{
    WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlBuffer, WebGlVertexArrayObject
};
use js_sys::{Float32Array, Object, Reflect, Array};
use wasm_bindgen::JsCast;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Log functions for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

// Constants
const DEFAULT_ELEMENT_COUNT: usize = 10;
const SEGMENTS: usize = 30; // Number of segments to approximate a shape
const MIN_POLYGON_SIDES: usize = 3;
const VERTICES_PER_POLYGON: usize = SEGMENTS + 2; // Center point + segments + repeat first point

// Shader sources
const VERTEX_SHADER_SRC: &str = r#"#version 300 es
in vec4 position;
in vec4 color;
in float instanceIndex;
in float sideCount;

uniform mat4 uMatrix;
uniform float uTime;
uniform float uAspectRatio;
uniform int uElementCount;

out vec4 vColor;
out float vDistFromCenter;

void main() {
    // Get shape index from the instance data
    float index = instanceIndex;
    float totalElements = float(uElementCount);
    
    // Calculate horizontal positioning to center the elements
    // This ranges from -1.0 to 1.0 for the entire row
    float xOffset = ((index * 2.0) / (totalElements - 1.0)) - 1.0;
    
    // Calculate vertical offset with sine wave and phase shift
    float phaseOffset = index * 0.5; // offset each polygon in the wave
    float yOffset = sin(uTime * 1.5 + phaseOffset) * 0.25;
    
    // Add a small horizontal movement 
    float xWobble = sin(uTime * 0.7 + phaseOffset * 1.3) * 0.02;
    
    // Add a bit of rotation to each polygon
    float rotationAngle = sin(uTime * 0.3 + phaseOffset) * 0.2;
    float cosVal = cos(rotationAngle);
    float sinVal = sin(rotationAngle);
    
    // Apply transformations to create a perfectly proportioned shape
    vec4 offsetPosition = position;
    
    // Apply slight rotation to each polygon
    float originalX = offsetPosition.x;
    float originalY = offsetPosition.y;
    offsetPosition.x = originalX * cosVal - originalY * sinVal;
    offsetPosition.y = originalX * sinVal + originalY * cosVal;
    
    // First apply the polygon scaling
    float baseScale = 0.15;
    
    // Make the polygons perfectly proportioned by applying aspect ratio correction
    if (uAspectRatio >= 1.0) {
        // Wide screen - correct the x coordinate
        offsetPosition.x *= baseScale; 
        offsetPosition.y *= baseScale;
    } else {
        // Tall screen - correct the y coordinate
        offsetPosition.x *= baseScale;
        offsetPosition.y *= baseScale * uAspectRatio;
    }
    
    // Then add the positional offsets - use xOffset directly for centered row
    offsetPosition.x += xOffset + xWobble;
    offsetPosition.y += yOffset;
    
    // Apply aspect ratio correction to maintain position spacing
    if (uAspectRatio >= 1.0) {
        // Wide screen
        offsetPosition.x /= uAspectRatio;
    } else {
        // Tall screen - already handled
    }
    
    // Set the final position
    gl_Position = uMatrix * offsetPosition;
    
    // Pass color to fragment shader
    vColor = color;
    
    // Calculate distance from center for fragment shader effects
    vDistFromCenter = length(position.xy) / 0.12; // Normalized distance
}
"#;

// Fragment shader source
const FRAGMENT_SHADER_SRC: &str = r#"#version 300 es
precision highp float;

in vec4 vColor;
in float vDistFromCenter;
uniform float uTime;

out vec4 outColor;

void main() {
    // Add subtle color pulsing effect
    float pulse = sin(uTime * 1.5) * 0.15 + 0.85;
    
    // Add time-based shimmer
    float shimmer = sin(uTime * 3.0 + vDistFromCenter * 3.0) * 0.1 + 0.9;
    
    // Combine effects but keep solid colors
    vec3 finalColor = vColor.rgb * pulse * shimmer;
    
    // Use full opacity for solid colors
    outColor = vec4(finalColor, 1.0);
}
"#;

// Shape type enum for JavaScript
#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum ShapeType {
    Regular,
    Star,
    Spiral,
}

// Options for rendering
#[wasm_bindgen]
pub struct RenderOptions {
    pub animate: bool,
    pub center_x: f32,
    pub center_y: f32,
    pub scale: f32,
    pub spacing: f32,
    pub rotation: f32,
    pub shape_type: ShapeType,
}

#[wasm_bindgen]
impl RenderOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RenderOptions {
        RenderOptions {
            animate: true,
            center_x: 0.0,
            center_y: 0.0,
            scale: 1.0,
            spacing: 1.0,
            rotation: 0.0,
            shape_type: ShapeType::Regular,
        }
    }
}

// Main Canvas2D GPU Renderer
#[wasm_bindgen]
pub struct Canvas2D {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    vao: WebGlVertexArrayObject,
    time_location: Option<web_sys::WebGlUniformLocation>,
    matrix_location: Option<web_sys::WebGlUniformLocation>,
    aspect_ratio_location: Option<web_sys::WebGlUniformLocation>,
    element_count_location: Option<web_sys::WebGlUniformLocation>,
    start_time: f64,
    last_frame_time: f64,
    width: u32,
    height: u32,
    element_count: u32,
    is_disposed: bool,
}

#[wasm_bindgen]
impl Canvas2D {
    // Initialize a new Canvas2D context
    #[wasm_bindgen]
    pub fn init(gl: WebGl2RenderingContext, width: u32, height: u32) -> Result<Canvas2D, JsValue> {
        console_log!("Initializing Canvas2D GPU Renderer");
        
        // Compile shaders and create program
        let vert_shader = compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, VERTEX_SHADER_SRC)?;
        let frag_shader = compile_shader(&gl, WebGl2RenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER_SRC)?;
        let program = link_program(&gl, &vert_shader, &frag_shader)?;
        
        // Use the program
        gl.use_program(Some(&program));
        
        // Get uniform locations
        let time_location = gl.get_uniform_location(&program, "uTime");
        let matrix_location = gl.get_uniform_location(&program, "uMatrix");
        let aspect_ratio_location = gl.get_uniform_location(&program, "uAspectRatio");
        let element_count_location = gl.get_uniform_location(&program, "uElementCount");
        
        // Create and bind VAO
        let vao = gl.create_vertex_array().ok_or("Failed to create vertex array")?;
        gl.bind_vertex_array(Some(&vao));
        
        // Setup viewport
        gl.viewport(0, 0, width as i32, height as i32);
        
        // Enable alpha blending
        gl.enable(WebGl2RenderingContext::BLEND);
        gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA);
        
        // Get initial time
        let performance = web_sys::window().unwrap().performance().unwrap();
        let start_time = performance.now();
        
        Ok(Canvas2D {
            gl,
            program,
            vao,
            time_location,
            matrix_location,
            aspect_ratio_location,
            element_count_location,
            start_time,
            last_frame_time: start_time,
            width,
            height,
            element_count: 0,
            is_disposed: false,
        })
    }
    
    // Draw polygons with increasing sides
    #[wasm_bindgen]
    pub fn draw_polygon_row(&mut self, count: u32, options: JsValue) -> Result<(), JsValue> {
        if self.is_disposed {
            return Err(JsValue::from_str("Canvas has been disposed"));
        }
        
        // Use default options if options is null or undefined
        // We're taking the options as JsValue instead of &RenderOptions
        // This provides more flexibility with what JavaScript can pass in
        
        // Setup the buffers for the polygons
        setup_polygon_buffers(&self.gl, &self.program, count as usize)?;
        self.element_count = count;
        
        Ok(())
    }
    
    // Clear the canvas
    #[wasm_bindgen]
    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) -> Result<(), JsValue> {
        if self.is_disposed {
            return Err(JsValue::from_str("Canvas has been disposed"));
        }
        
        self.gl.clear_color(r, g, b, a);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        
        Ok(())
    }
    
    // Render a frame
    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<f64, JsValue> {
        if self.is_disposed {
            return Err(JsValue::from_str("Canvas has been disposed"));
        }
        
        if self.element_count == 0 {
            return Ok(0.0); // Nothing to render
        }
        
        // Get current time
        let performance = web_sys::window().unwrap().performance().unwrap();
        let current_time = performance.now();
        let elapsed = (current_time - self.start_time) / 1000.0; // Convert to seconds
        let delta_time = (current_time - self.last_frame_time) / 1000.0;
        self.last_frame_time = current_time;
        
        // Use our shader program
        self.gl.use_program(Some(&self.program));
        
        // Bind VAO
        self.gl.bind_vertex_array(Some(&self.vao));
        
        // Update time uniform
        if let Some(time_loc) = &self.time_location {
            self.gl.uniform1f(Some(time_loc), elapsed as f32);
        }
        
        // Calculate aspect ratio
        let aspect_ratio = self.width as f32 / self.height as f32;
        
        // Update aspect ratio uniform
        if let Some(aspect_ratio_loc) = &self.aspect_ratio_location {
            self.gl.uniform1f(Some(aspect_ratio_loc), aspect_ratio);
        }
        
        // Update element count uniform
        if let Some(element_count_loc) = &self.element_count_location {
            self.gl.uniform1i(Some(element_count_loc), self.element_count as i32);
        }
        
        // Create and update projection matrix
        if let Some(matrix_loc) = &self.matrix_location {
            // Just identity matrix for now
            let identity = [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ];
            self.gl.uniform_matrix4fv_with_f32_array(Some(matrix_loc), false, &identity);
        }
        
        // Clear the canvas with a nice gradient-like dark background
        let bg_time = (elapsed * 0.1).sin() * 0.02 + 0.05;
        self.gl.clear_color(bg_time as f32, bg_time as f32 * 0.8, bg_time as f32 * 1.2, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        
        // Draw the polygons
        let vertices_per_instance = VERTICES_PER_POLYGON as i32;
        
        for i in 0..self.element_count {
            self.gl.draw_arrays(
                WebGl2RenderingContext::TRIANGLE_FAN,
                (i as i32) * vertices_per_instance,
                vertices_per_instance
            );
        }
        
        Ok(delta_time * 1000.0) // Return the delta time in milliseconds
    }
    
    // Resize the canvas
    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        if self.is_disposed {
            return Err(JsValue::from_str("Canvas has been disposed"));
        }
        
        self.width = width;
        self.height = height;
        self.gl.viewport(0, 0, width as i32, height as i32);
        
        Ok(())
    }
    
    // Get current element count
    #[wasm_bindgen]
    pub fn get_element_count(&self) -> u32 {
        self.element_count
    }
    
    // Dispose of resources
    #[wasm_bindgen]
    pub fn dispose(&mut self) -> Result<(), JsValue> {
        if self.is_disposed {
            return Ok(());
        }
        
        // Delete WebGL resources
        self.gl.delete_program(Some(&self.program));
        self.gl.delete_vertex_array(Some(&self.vao));
        
        self.is_disposed = true;
        console_log!("Canvas2D GPU Renderer disposed");
        
        Ok(())
    }
}

// Setup buffers for polygons with increasing number of sides
fn setup_polygon_buffers(gl: &WebGl2RenderingContext, program: &WebGlProgram, num_polygons: usize) -> Result<(), JsValue> {
    // Create vertices for all polygons
    let mut all_vertices = Vec::with_capacity(num_polygons * VERTICES_PER_POLYGON * 3);
    let mut all_colors = Vec::with_capacity(num_polygons * VERTICES_PER_POLYGON * 4);
    let mut all_instance_indices = Vec::with_capacity(num_polygons * VERTICES_PER_POLYGON);
    let mut all_side_counts = Vec::with_capacity(num_polygons * VERTICES_PER_POLYGON);

    // For each polygon
    for i in 0..num_polygons {
        // Create a polygon with i+3 sides (start with triangle)
        let sides = MIN_POLYGON_SIDES + i;
        add_polygon_vertices(&mut all_vertices, sides);
        
        // Set color for this polygon (RGB rainbow distribution)
        let hue = (i as f32) / (num_polygons as f32);
        let saturation = 0.9; // Slightly more vibrant
        let lightness = 0.6;  // Slightly brighter
        let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);
        
        // Add color for all vertices of this polygon
        for _ in 0..VERTICES_PER_POLYGON {
            all_colors.push(r);
            all_colors.push(g);
            all_colors.push(b);
            all_colors.push(1.0); // Alpha
        }
        
        // Add instance index for all vertices of this polygon
        for _ in 0..VERTICES_PER_POLYGON {
            all_instance_indices.push(i as f32);
            all_side_counts.push(sides as f32);
        }
    }
    
    // Create and bind position buffer
    let position_buffer = gl.create_buffer().ok_or("Failed to create position buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));
    
    // Pass the vertices to WebGL
    let positions_array = Float32Array::from(&all_vertices[..]);
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &positions_array,
        WebGl2RenderingContext::STATIC_DRAW,
    );
    
    // Setup position attribute
    let position_attr_location = gl.get_attrib_location(&program, "position") as u32;
    gl.vertex_attrib_pointer_with_i32(
        position_attr_location,
        3,                     // 3 components per vertex (x, y, z)
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(position_attr_location);
    
    // Create and bind color buffer
    let color_buffer = gl.create_buffer().ok_or("Failed to create color buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));
    
    // Pass the colors to WebGL
    let colors_array = Float32Array::from(&all_colors[..]);
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &colors_array,
        WebGl2RenderingContext::STATIC_DRAW,
    );
    
    // Setup color attribute
    let color_attr_location = gl.get_attrib_location(&program, "color") as u32;
    gl.vertex_attrib_pointer_with_i32(
        color_attr_location,
        4,                     // 4 components per vertex (r, g, b, a)
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(color_attr_location);
    
    // Create and bind instance index buffer
    let instance_buffer = gl.create_buffer().ok_or("Failed to create instance buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&instance_buffer));
    
    // Pass the instance indices to WebGL
    let instance_array = Float32Array::from(&all_instance_indices[..]);
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &instance_array,
        WebGl2RenderingContext::STATIC_DRAW,
    );
    
    // Setup instance index attribute
    let instance_attr_location = gl.get_attrib_location(&program, "instanceIndex") as u32;
    gl.vertex_attrib_pointer_with_i32(
        instance_attr_location,
        1,                     // 1 component per vertex (index)
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(instance_attr_location);
    
    // Create and bind side count buffer
    let side_count_buffer = gl.create_buffer().ok_or("Failed to create side count buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&side_count_buffer));
    
    // Pass the side counts to WebGL
    let side_count_array = Float32Array::from(&all_side_counts[..]);
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &side_count_array,
        WebGl2RenderingContext::STATIC_DRAW,
    );
    
    // Setup side count attribute
    let side_count_attr_location = gl.get_attrib_location(&program, "sideCount") as u32;
    gl.vertex_attrib_pointer_with_i32(
        side_count_attr_location,
        1,                     // 1 component per vertex (side count)
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(side_count_attr_location);
    
    Ok(())
}

// Add vertices for a polygon with the specified number of sides
fn add_polygon_vertices(vertices: &mut Vec<f32>, sides: usize) {
    // Center point
    vertices.push(0.0); // x
    vertices.push(0.0); // y
    vertices.push(0.0); // z
    
    // Use actual number of sides or cap at SEGMENTS
    let actual_sides = sides.min(SEGMENTS);
    
    // Generate points around the polygon
    for i in 0..=actual_sides {
        let angle = (i % actual_sides) as f32 * 2.0 * std::f32::consts::PI / (actual_sides as f32);
        let x = angle.cos() * 0.12; // Radius 0.12
        let y = angle.sin() * 0.12;
        
        vertices.push(x);
        vertices.push(y);
        vertices.push(0.0);
    }
    
    // Fill the rest with the last point to match VERTICES_PER_POLYGON
    let last_x = vertices[vertices.len() - 3];
    let last_y = vertices[vertices.len() - 2];
    
    for _ in (actual_sides + 1)..=SEGMENTS {
        vertices.push(last_x);
        vertices.push(last_y);
        vertices.push(0.0);
    }
}

// Convert HSL color to RGB
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        // Achromatic (gray)
        return (l, l, l);
    }
    
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0)
    )
}

// Helper function for HSL to RGB conversion
fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    
    p
}

// Helper function to compile a shader
fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    
    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

// Helper function to link a shader program
fn link_program(
    gl: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader program"))?;
    
    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);
    
    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program")))
    }
} 