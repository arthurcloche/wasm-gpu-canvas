import { CanvasGPU, wasmModule } from "/src/canvas-gpu.js";
import FpsCounter from "./fps-counter.js";

/**
 * Create and initialize an example container with canvas
 * @param {string} title - Example title
 * @param {string} description - Example description
 * @param {Object} options - Canvas options
 * @returns {Object} - Example context with canvas, container and canvasGPU instance
 */
export async function createExample(title, description, options = {}) {
  const defaults = {
    width: 600,
    height: 400,
    parentSelector: "body",
  };

  const config = { ...defaults, ...options };

  // Create container
  const container = document.createElement("div");
  container.className = "example-container";
  container.style.margin = "20px auto";
  container.style.maxWidth = `${config.width}px`;
  container.style.position = "relative";

  // Create title
  const titleEl = document.createElement("h2");
  titleEl.textContent = title;
  titleEl.style.margin = "0 0 10px 0";
  titleEl.style.fontFamily = "system-ui, sans-serif";

  // Create canvas
  const canvas = document.createElement("canvas");
  canvas.width = config.width;
  canvas.height = config.height;
  canvas.style.display = "block";
  canvas.style.width = "100%";
  canvas.style.height = "auto";
  canvas.style.background = "#111";

  // Create description
  const descEl = document.createElement("p");
  descEl.textContent = description;
  descEl.style.margin = "10px 0 0 0";
  descEl.style.fontFamily = "system-ui, sans-serif";
  descEl.style.fontSize = "14px";
  descEl.style.color = "#333";

  // Assemble DOM
  container.appendChild(titleEl);
  container.appendChild(canvas);
  container.appendChild(descEl);

  // Add to page
  const parent = document.querySelector(config.parentSelector);
  parent.appendChild(container);

  // Create FPS counter
  const fpsCounter = new FpsCounter();
  fpsCounter.attach(container);

  // Initialize CanvasGPU
  const canvasGPU = new CanvasGPU();

  try {
    // Wait for WASM to initialize first
    await wasmModule.init();

    // Then initialize CanvasGPU with the canvas
    await canvasGPU.init(canvas, {
      animate: true,
    });
  } catch (err) {
    console.error("Error initializing example:", err);
    descEl.textContent = `Error: ${err.message}`;
    descEl.style.color = "red";
    return { canvas, container, error: err };
  }

  // Animation loop wrapper
  const animate = (renderFn) => {
    let running = true;

    const loop = () => {
      if (!running) return;

      // Update FPS counter
      fpsCounter.update();

      // Call the render function
      renderFn();

      // Request next frame
      requestAnimationFrame(loop);
    };

    // Start the loop
    loop();

    // Return a function to stop animation
    return () => {
      running = false;
    };
  };

  return { canvas, container, canvasGPU, animate, fpsCounter };
}

/**
 * Initialize examples page with basic styling
 */
export function initExamplesPage(title = "WebGL2 Canvas Examples") {
  // Add basic page styling
  const style = document.createElement("style");
  style.textContent = `
    body {
      font-family: system-ui, -apple-system, sans-serif;
      background: #f5f5f5;
      color: #333;
      margin: 0;
      padding: 20px;
    }
    
    h1 {
      margin: 0 0 20px 0;
      text-align: center;
    }
    
    .example-container {
      background: white;
      border-radius: 8px;
      padding: 20px;
      box-shadow: 0 2px 10px rgba(0,0,0,0.1);
      margin-bottom: 30px;
    }
  `;
  document.head.appendChild(style);

  // Add title
  const heading = document.createElement("h1");
  heading.textContent = title;
  document.body.appendChild(heading);
}

/**
 * Initialize multiple examples in a grid layout
 */
export function initExamplesGrid(columns = 2) {
  initExamplesPage();

  // Create grid container
  const grid = document.createElement("div");
  grid.className = "examples-grid";
  grid.style.display = "grid";
  grid.style.gridTemplateColumns = `repeat(${columns}, 1fr)`;
  grid.style.gap = "20px";

  document.body.appendChild(grid);

  return grid;
}
