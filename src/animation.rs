use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::{GpuBuffer, GpuDevice};

// Structure to handle animation state
pub struct AnimationState {
    time: f32,
    buffer: GpuBuffer,
    bind_group: Object,
}

impl AnimationState {
    pub fn new(device: &GpuDevice) -> Result<Self, JsValue> {
        // Create a buffer to store time uniform
        let buffer = Self::create_time_buffer(device)?;
        
        // Create a bind group for the time uniform
        let bind_group = Self::create_bind_group(device, &buffer)?;
        
        Ok(Self {
            time: 0.0,
            buffer,
            bind_group,
        })
    }
    
    // Create a uniform buffer for time
    fn create_time_buffer(device: &GpuDevice) -> Result<GpuBuffer, JsValue> {
        let buffer_desc = Object::new();
        
        // Set buffer size (4 bytes for a single float)
        Reflect::set(&buffer_desc, &JsValue::from_str("size"), &JsValue::from_f64(4.0))?;
        
        // Set buffer usage
        Reflect::set(
            &buffer_desc, 
            &JsValue::from_str("usage"), 
            &JsValue::from_f64((1 << 0) | (1 << 1))  // UNIFORM | COPY_DST
        )?;
        
        // Set buffer label
        Reflect::set(&buffer_desc, &JsValue::from_str("label"), &JsValue::from_str("Time Uniform Buffer"))?;
        
        let buffer = device.create_buffer(&buffer_desc);
        Ok(buffer)
    }
    
    // Create a bind group for our uniform buffer
    fn create_bind_group(device: &GpuDevice, buffer: &GpuBuffer) -> Result<Object, JsValue> {
        // Create bind group layout
        let bind_group_layout_desc = Object::new();
        
        // Define entries
        let entries = Array::new();
        let entry = Object::new();
        
        // Configure entry binding
        Reflect::set(&entry, &JsValue::from_str("binding"), &JsValue::from_f64(0.0))?;
        Reflect::set(&entry, &JsValue::from_str("visibility"), &JsValue::from_f64(1 | 2))?; // VERTEX | FRAGMENT
        
        // Configure buffer binding layout
        let buffer_binding = Object::new();
        Reflect::set(&buffer_binding, &JsValue::from_str("type"), &JsValue::from_str("uniform"))?;
        Reflect::set(&entry, &JsValue::from_str("buffer"), &buffer_binding)?;
        
        // Add entry to entries
        entries.push(&entry);
        
        // Add entries to layout descriptor
        Reflect::set(&bind_group_layout_desc, &JsValue::from_str("entries"), &entries)?;
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_desc);
        
        // Create bind group
        let bind_group_desc = Object::new();
        Reflect::set(&bind_group_desc, &JsValue::from_str("layout"), &bind_group_layout)?;
        
        // Create bind group entries
        let bg_entries = Array::new();
        let bg_entry = Object::new();
        
        Reflect::set(&bg_entry, &JsValue::from_str("binding"), &JsValue::from_f64(0.0))?;
        
        // Create resource object for the buffer
        let resource = Object::new();
        Reflect::set(&resource, &JsValue::from_str("buffer"), buffer)?;
        Reflect::set(&bg_entry, &JsValue::from_str("resource"), &resource)?;
        
        bg_entries.push(&bg_entry);
        Reflect::set(&bind_group_desc, &JsValue::from_str("entries"), &bg_entries)?;
        
        let bind_group = device.create_bind_group(&bind_group_desc);
        Ok(bind_group)
    }
    
    // Update animation state
    pub fn update(&mut self, device: &GpuDevice, delta_time: f32) {
        self.time += delta_time;
        
        // Write new time to buffer
        let array = js_sys::Float32Array::new_with_length(1);
        array.set_index(0, self.time);
        
        // Get device queue
        let queue = device.queue();
        
        // Write to buffer
        queue.write_buffer_with_u32_and_buffer_source(
            &self.buffer,
            0,
            &array.buffer(),
            array.byte_offset(),
            array.byte_length(),
        );
    }
    
    // Get bind group
    pub fn get_bind_group(&self) -> &Object {
        &self.bind_group
    }
} 