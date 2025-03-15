# WASM WebGL2-Accelerated Canvas

This project demonstrates how to use WebAssembly (WASM) with WebGL2 to create GPU-accelerated 2D canvas rendering.

## Prerequisites

- Rust and Cargo installed (https://rustup.rs/)
- wasm-pack installed (`cargo install wasm-pack`)
- Node.js and npm installed (https://nodejs.org/)
- A browser that supports WebGL2 (most modern browsers)

## Setup

1. Clone this repository
2. Install dependencies:
   ```
   npm install
   ```
3. Build the project:
   ```
   npm run build
   ```
4. Start the development server:
   ```
   npm start
   ```

## Project Structure

- `www/` - JavaScript and HTML frontend code
- `src/` - Rust code for the WASM module
- `examples/` - Creative coding examples using the library
- `Cargo.toml` - Rust dependencies
- `package.json` - npm dependencies
- `webpack.config.js` - Webpack configuration

## How It Works

This project demonstrates how to:

1. Create a WebGL2 context for a canvas element
2. Set up a GPU rendering pipeline in Rust
3. Compile Rust code to WASM
4. Execute GPU-accelerated rendering operations from JavaScript

WebGL2 provides direct access to the GPU, allowing for much more efficient rendering than the traditional 2D Canvas API. This approach works in all modern browsers, providing excellent compatibility while still leveraging GPU acceleration.

## Features

- GPU-accelerated polygon rendering with WebGL2
- Web Component API (`<canvas-gpu>`) for easy integration
- JavaScript API for programmatic control
- Animated effects using GLSL shaders
- Dynamic controls for polygon count and animation settings
- Responsive design that adapts to window resizing

## Examples

The project includes several creative coding examples in the style of Processing/p5.js:

1. **Particle System** - Thousands of interactive particles with physics simulation
2. **Voronoi Patterns** - Dynamic Voronoi diagram with interactive cells
3. **Cellular Automata** - Conway's Game of Life with interactive cells
4. **Fractal Tree** - Recursive branching patterns that grow and evolve
5. **Flow Field** - Particles following vector fields with emergent patterns
6. **Interactive Polygons** - Mouse-reactive shapes with geometric transformations
7. **Color Mixing** - Gradient experiments and color theory visualization
8. **Audio Visualizer** - Reactive visuals responding to audio input
9. **Spiral Generator** - Mathematical patterns with configurable parameters
10. **Pendulum Simulation** - Physics-based animation of connected pendulums

Each example demonstrates different capabilities of the WebGL2 Canvas library and includes an FPS counter to monitor performance.

## Usage

### Web Component API

```html
<canvas-gpu width="800" height="400" count="10"></canvas-gpu>
```

Attributes:
- `width` - Canvas width in pixels
- `height` - Canvas height in pixels
- `count` - Number of elements to render
- `animate` - Enable/disable animation (true/false)
- `scale` - Scale factor for elements
- `spacing` - Spacing between elements
- `rotation` - Base rotation in radians

### JavaScript API

```javascript
import { CanvasGPU } from "./canvas-gpu.js";

// Create instance
const canvasGPU = new CanvasGPU();

// Initialize with canvas
await canvasGPU.init(document.getElementById('canvas'), {
  animate: true,
  scale: 1.0
});

// Draw elements
canvasGPU.drawPolygonRow(10);

// Start animation
canvasGPU.start();
```

## Further Exploration

- Try modifying the shaders in `src/lib.rs` to create different visual effects
- Experiment with more complex geometries and rendering techniques
- Add textures and more advanced rendering features
- Implement custom controls for interactive elements
- Compare performance with traditional Canvas 2D rendering
