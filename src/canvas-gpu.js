import init, {
  Canvas2D,
  RenderOptions,
  ShapeType,
} from "../pkg/wasm_2dcanvas_gpu.js";

// Initialize WASM
const wasmModule = {
  isInitialized: false,
  initPromise: null,

  init: function () {
    if (!this.initPromise) {
      this.initPromise = init()
        .then(() => {
          console.log("WebAssembly module initialized successfully");
          this.isInitialized = true;
          return true;
        })
        .catch((err) => {
          console.error("Failed to initialize WebAssembly module:", err);
          return Promise.reject(err);
        });
    }
    return this.initPromise;
  },
};

/**
 * CanvasGPU - A WebGL2-accelerated 2D canvas API
 * Similar to the native CanvasRenderingContext2D but using WebGL2 for acceleration
 */
class CanvasGPU {
  constructor() {
    this.canvas = null;
    this.gl = null;
    this.renderer = null;
    this.animationFrameId = null;
    this.isReady = false;
    this.options = null;
    this.elementCount = 10; // default
  }

  /**
   * Initialize the CanvasGPU with a canvas element
   * @param {HTMLCanvasElement} canvas - The canvas element to render to
   * @param {Object} options - Options for initialization
   * @returns {Promise} - Resolves when initialized
   */
  async init(canvas, options = {}) {
    this.canvas = canvas;

    // Get WebGL2 context
    this.gl = canvas.getContext("webgl2");

    if (!this.gl) {
      throw new Error("WebGL2 is not supported in your browser");
    }

    // Wait for WASM to initialize
    await wasmModule.init();

    try {
      // Create options object manually instead of using constructor
      this.options = {};
      this.options.animate =
        options.animate !== undefined ? options.animate : true;
      this.options.center_x =
        options.centerX !== undefined ? options.centerX : 0.0;
      this.options.center_y =
        options.centerY !== undefined ? options.centerY : 0.0;
      this.options.scale = options.scale !== undefined ? options.scale : 1.0;
      this.options.spacing =
        options.spacing !== undefined ? options.spacing : 1.0;
      this.options.rotation =
        options.rotation !== undefined ? options.rotation : 0.0;
      this.options.shape_type =
        options.shapeType !== undefined ? options.shapeType : 0; // Regular

      // Create renderer
      this.renderer = Canvas2D.init(this.gl, canvas.width, canvas.height);

      // Draw initial shapes
      this.drawPolygonRow(this.elementCount);

      this.isReady = true;
    } catch (error) {
      console.error("Error during initialization:", error);
      throw error;
    }

    return this;
  }

  /**
   * Draw a row of polygons with increasing sides
   * @param {number} count - Number of polygons to draw
   * @param {Object} options - Options for drawing
   * @returns {CanvasGPU} - For chaining
   */
  drawPolygonRow(count, options = {}) {
    if (!this.isReady || !this.renderer) {
      console.warn("CanvasGPU not initialized yet");
      return this;
    }

    // Update options if provided
    if (options.animate !== undefined) this.options.animate = options.animate;
    if (options.centerX !== undefined) this.options.center_x = options.centerX;
    if (options.centerY !== undefined) this.options.center_y = options.centerY;
    if (options.scale !== undefined) this.options.scale = options.scale;
    if (options.spacing !== undefined) this.options.spacing = options.spacing;
    if (options.rotation !== undefined)
      this.options.rotation = options.rotation;
    if (options.shapeType !== undefined)
      this.options.shape_type = options.shapeType;

    this.elementCount = count;

    // Pass a JavaScript object directly for options
    this.renderer.draw_polygon_row(count, this.options);

    return this;
  }

  /**
   * Clear the canvas with a specified color
   * @param {number} r - Red (0-1)
   * @param {number} g - Green (0-1)
   * @param {number} b - Blue (0-1)
   * @param {number} a - Alpha (0-1)
   * @returns {CanvasGPU} - For chaining
   */
  clear(r = 0, g = 0, b = 0.1, a = 1) {
    if (!this.isReady || !this.renderer) {
      console.warn("CanvasGPU not initialized yet");
      return this;
    }

    this.renderer.clear(r, g, b, a);
    return this;
  }

  /**
   * Resize the canvas
   * @param {number} width - New width
   * @param {number} height - New height
   * @returns {CanvasGPU} - For chaining
   */
  resize(width, height) {
    if (!this.isReady || !this.renderer) {
      console.warn("CanvasGPU not initialized yet");
      return this;
    }

    this.canvas.width = width;
    this.canvas.height = height;
    this.renderer.resize(width, height);

    return this;
  }

  /**
   * Start the animation loop
   * @returns {CanvasGPU} - For chaining
   */
  start() {
    if (!this.isReady || !this.renderer) {
      console.warn("CanvasGPU not initialized yet");
      return this;
    }

    const animate = () => {
      try {
        this.renderer.render();
        this.animationFrameId = requestAnimationFrame(animate);
      } catch (e) {
        console.error("Render error:", e);
        this.stop();
      }
    };

    animate();
    return this;
  }

  /**
   * Stop the animation loop
   * @returns {CanvasGPU} - For chaining
   */
  stop() {
    if (this.animationFrameId) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }

    return this;
  }

  /**
   * Dispose of the renderer and release resources
   */
  dispose() {
    this.stop();

    if (this.renderer) {
      this.renderer.dispose();
      this.renderer = null;
    }

    this.isReady = false;
  }
}

export { CanvasGPU, wasmModule, ShapeType };
