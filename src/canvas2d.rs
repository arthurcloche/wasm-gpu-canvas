use js_sys::{Array, Float32Array, Object, Reflect, Uint16Array};
use wasm_bindgen::prelude::*;
use web_sys::{GpuBuffer, GpuDevice};

// Constants for buffer usage flags
const VERTEX_BUFFER: u32 = 1 << 3;
const INDEX_BUFFER: u32 = 1 << 4;
const COPY_DST: u32 = 1 << 1;

// Maximum vertices and indices
const MAX_VERTICES: usize = 10000;
const MAX_INDICES: usize = 15000;

// 2D Canvas-like Drawing Context
pub struct Canvas2D {
    // WebGPU resources
    device: GpuDevice,
    vertex_buffer: GpuBuffer,
    index_buffer: GpuBuffer,
    
    // Dynamic geometry data
    vertices: Vec<f32>,
    indices: Vec<u16>,
    
    // Current state
    vertex_count: usize,
    index_count: usize,
    current_color: [f32; 4],
}

impl Canvas2D {
    pub fn new(device: &GpuDevice) -> Result<Self, JsValue> {
        // Create vertex buffer (position, color)
        let vertex_buffer = Self::create_vertex_buffer(device, MAX_VERTICES * 6)?; // 2 for position, 4 for color
        
        // Create index buffer
        let index_buffer = Self::create_index_buffer(device, MAX_INDICES)?;
        
        Ok(Self {
            device: device.clone(),
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(MAX_VERTICES * 6),
            indices: Vec::with_capacity(MAX_INDICES),
            vertex_count: 0,
            index_count: 0,
            current_color: [1.0, 1.0, 1.0, 1.0], // White by default
        })
    }
    
    // Create the vertex buffer
    fn create_vertex_buffer(device: &GpuDevice, size: usize) -> Result<GpuBuffer, JsValue> {
        let buffer_desc = Object::new();
        
        // Set buffer size (6 floats per vertex: x, y, r, g, b, a)
        let byte_size = (size * std::mem::size_of::<f32>()) as f64;
        Reflect::set(&buffer_desc, &JsValue::from_str("size"), &JsValue::from_f64(byte_size))?;
        
        // Set buffer usage
        Reflect::set(
            &buffer_desc, 
            &JsValue::from_str("usage"), 
            &JsValue::from_f64((VERTEX_BUFFER | COPY_DST) as f64)
        )?;
        
        // Set buffer label
        Reflect::set(&buffer_desc, &JsValue::from_str("label"), &JsValue::from_str("Canvas2D Vertex Buffer"))?;
        
        let buffer = device.create_buffer(&buffer_desc);
        Ok(buffer)
    }
    
    // Create the index buffer
    fn create_index_buffer(device: &GpuDevice, size: usize) -> Result<GpuBuffer, JsValue> {
        let buffer_desc = Object::new();
        
        // Set buffer size (u16 per index)
        let byte_size = (size * std::mem::size_of::<u16>()) as f64;
        Reflect::set(&buffer_desc, &JsValue::from_str("size"), &JsValue::from_f64(byte_size))?;
        
        // Set buffer usage
        Reflect::set(
            &buffer_desc, 
            &JsValue::from_str("usage"), 
            &JsValue::from_f64((INDEX_BUFFER | COPY_DST) as f64)
        )?;
        
        // Set buffer label
        Reflect::set(&buffer_desc, &JsValue::from_str("label"), &JsValue::from_str("Canvas2D Index Buffer"))?;
        
        let buffer = device.create_buffer(&buffer_desc);
        Ok(buffer)
    }
    
    // Set the current drawing color
    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.current_color = [r, g, b, a];
    }
    
    // Draw a rectangle
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // Add vertices for rectangle (4 corners)
        let base_vertex = self.vertex_count as u16;
        
        // Top-left
        self.add_vertex(x, y);
        
        // Top-right
        self.add_vertex(x + width, y);
        
        // Bottom-right
        self.add_vertex(x + width, y + height);
        
        // Bottom-left
        self.add_vertex(x, y + height);
        
        // Add indices for two triangles
        self.indices.push(base_vertex);
        self.indices.push(base_vertex + 1);
        self.indices.push(base_vertex + 2);
        
        self.indices.push(base_vertex);
        self.indices.push(base_vertex + 2);
        self.indices.push(base_vertex + 3);
        
        self.index_count += 6;
    }
    
    // Draw a circle
    pub fn fill_circle(&mut self, x: f32, y: f32, radius: f32, segments: u32) {
        let base_vertex = self.vertex_count as u16;
        
        // Center vertex
        self.add_vertex(x, y);
        
        // Edge vertices
        for i in 0..segments {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
            let vx = x + radius * angle.cos();
            let vy = y + radius * angle.sin();
            self.add_vertex(vx, vy);
        }
        
        // Add indices for triangle fan
        for i in 0..segments {
            self.indices.push(base_vertex); // Center
            self.indices.push(base_vertex + 1 + i as u16); // Current edge vertex
            self.indices.push(base_vertex + 1 + ((i + 1) % segments) as u16); // Next edge vertex
            self.index_count += 3;
        }
    }
    
    // Draw a line with thickness
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32) {
        let base_vertex = self.vertex_count as u16;
        
        // Calculate perpendicular direction
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();
        
        if length < 0.0001 {
            return; // Too short to draw
        }
        
        let nx = dy / length * thickness * 0.5;
        let ny = -dx / length * thickness * 0.5;
        
        // Add vertices for line quad
        self.add_vertex(x1 + nx, y1 + ny); // Top-left
        self.add_vertex(x2 + nx, y2 + ny); // Top-right
        self.add_vertex(x2 - nx, y2 - ny); // Bottom-right
        self.add_vertex(x1 - nx, y1 - ny); // Bottom-left
        
        // Add indices for two triangles
        self.indices.push(base_vertex);
        self.indices.push(base_vertex + 1);
        self.indices.push(base_vertex + 2);
        
        self.indices.push(base_vertex);
        self.indices.push(base_vertex + 2);
        self.indices.push(base_vertex + 3);
        
        self.index_count += 6;
    }
    
    // Helper to add a vertex with the current color
    fn add_vertex(&mut self, x: f32, y: f32) {
        // Add position
        self.vertices.push(x);
        self.vertices.push(y);
        
        // Add color
        self.vertices.push(self.current_color[0]);
        self.vertices.push(self.current_color[1]);
        self.vertices.push(self.current_color[2]);
        self.vertices.push(self.current_color[3]);
        
        self.vertex_count += 1;
    }
    
    // Upload the buffered geometry to the GPU
    pub fn upload(&self) -> Result<(), JsValue> {
        // Upload vertices
        let vertex_array = Float32Array::new_with_length(self.vertices.len() as u32);
        for (i, value) in self.vertices.iter().enumerate() {
            vertex_array.set_index(i as u32, *value);
        }
        
        // Upload to GPU
        let queue = self.device.queue();
        queue.write_buffer_with_u32_and_buffer_source(
            &self.vertex_buffer,
            0,
            &vertex_array.buffer(),
            vertex_array.byte_offset(),
            vertex_array.byte_length(),
        );
        
        // Upload indices
        let index_array = Uint16Array::new_with_length(self.indices.len() as u32);
        for (i, value) in self.indices.iter().enumerate() {
            index_array.set_index(i as u32, *value);
        }
        
        // Upload to GPU
        queue.write_buffer_with_u32_and_buffer_source(
            &self.index_buffer,
            0,
            &index_array.buffer(),
            index_array.byte_offset(),
            index_array.byte_length(),
        );
        
        Ok(())
    }
    
    // Clear all buffered geometry
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.vertex_count = 0;
        self.index_count = 0;
    }
    
    // Get vertex buffer for rendering
    pub fn vertex_buffer(&self) -> &GpuBuffer {
        &self.vertex_buffer
    }
    
    // Get index buffer for rendering
    pub fn index_buffer(&self) -> &GpuBuffer {
        &self.index_buffer
    }
    
    // Get the number of indices to draw
    pub fn index_count(&self) -> usize {
        self.index_count
    }
    
    // Get the vertex stride (6 floats per vertex)
    pub fn vertex_stride() -> u64 {
        6 * std::mem::size_of::<f32>() as u64
    }
} 