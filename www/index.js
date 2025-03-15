import { CanvasGPU, wasmModule } from "../src/canvas-gpu.js";

/**
 * CanvasGPU Web Component
 * Provides a declarative way to use the CanvasGPU API
 */
class CanvasGpuElement extends HTMLElement {
  constructor() {
    super();
    this.canvasGPU = new CanvasGPU();
    this.shadow = this.attachShadow({ mode: "open" });
    this.canvas = null;
    this.isReady = false;
  }

  static get observedAttributes() {
    return [
      "width",
      "height",
      "count",
      "animate",
      "scale",
      "spacing",
      "rotation",
    ];
  }

  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue === newValue) return;

    if (!this.isReady) return;

    if (name === "width" || name === "height") {
      this.updateCanvasSize();
    } else if (name === "count") {
      const count = parseInt(newValue, 10) || 10;
      this.canvasGPU.drawPolygonRow(count);
    } else if (name === "animate") {
      const animate = newValue !== "false";
      this.canvasGPU.options.animate = animate;

      if (animate && !this.canvasGPU.animationFrameId) {
        this.canvasGPU.start();
      } else if (!animate && this.canvasGPU.animationFrameId) {
        this.canvasGPU.stop();
      }
    } else if (name === "scale") {
      this.canvasGPU.options.scale = parseFloat(newValue) || 1.0;
    } else if (name === "spacing") {
      this.canvasGPU.options.spacing = parseFloat(newValue) || 1.0;
    } else if (name === "rotation") {
      this.canvasGPU.options.rotation = parseFloat(newValue) || 0.0;
    }
  }

  connectedCallback() {
    this.initialize();
  }

  disconnectedCallback() {
    this.canvasGPU.dispose();
  }

  async initialize() {
    // Create canvas element
    this.canvas = document.createElement("canvas");
    const width = parseInt(this.getAttribute("width"), 10) || 800;
    const height = parseInt(this.getAttribute("height"), 10) || 600;
    const count = parseInt(this.getAttribute("count"), 10) || 10;

    this.canvas.width = width;
    this.canvas.height = height;
    this.canvas.style.display = "block";
    this.canvas.style.width = "100%";
    this.canvas.style.height = "auto";

    // Add canvas to shadow DOM
    this.shadow.appendChild(this.canvas);

    try {
      // Initialize CanvasGPU
      await this.canvasGPU.init(this.canvas, {
        animate: this.getAttribute("animate") !== "false",
        scale: parseFloat(this.getAttribute("scale")) || 1.0,
        spacing: parseFloat(this.getAttribute("spacing")) || 1.0,
        rotation: parseFloat(this.getAttribute("rotation")) || 0.0,
      });

      // Draw initial shapes
      this.canvasGPU.drawPolygonRow(count);

      // Start animation if animate is not explicitly set to false
      if (this.getAttribute("animate") !== "false") {
        this.canvasGPU.start();
      }

      this.isReady = true;

      // Update status
      const statusEl = document.getElementById("webgl-status");
      if (statusEl) {
        statusEl.textContent = `WebGL2 status: Running with ${count} polygons`;
      }
    } catch (error) {
      console.error("Error initializing CanvasGPU:", error);

      const errorEl = document.createElement("div");
      errorEl.textContent = `Error: ${error.message}`;
      errorEl.style.color = "red";
      errorEl.style.padding = "20px";
      this.shadow.appendChild(errorEl);

      const statusEl = document.getElementById("webgl-status");
      if (statusEl) {
        statusEl.textContent = "WebGL2 status: Error initializing";
      }
    }
  }

  updateCanvasSize() {
    if (!this.canvas || !this.canvasGPU) return;

    const width = parseInt(this.getAttribute("width"), 10) || 800;
    const height = parseInt(this.getAttribute("height"), 10) || 600;

    this.canvasGPU.resize(width, height);
  }
}

// Define the web component
customElements.define("canvas-gpu", CanvasGpuElement);

// Add event listeners once the DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
  const circleCountSlider = document.getElementById("circle-count");
  if (circleCountSlider) {
    circleCountSlider.addEventListener("input", (e) => {
      const value = e.target.value;
      document.getElementById("circle-count-value").textContent = value;
      document.querySelector("canvas-gpu").setAttribute("count", value);
    });
  }
});

console.log("Canvas GPU Library initialized");
