/**
 * FPS Counter - A simple utility to track and display frames per second
 */
class FpsCounter {
  constructor(updateInterval = 500) {
    this.fps = 0;
    this.frames = 0;
    this.lastUpdate = performance.now();
    this.updateInterval = updateInterval; // ms between updates
    this.element = null;
  }

  /**
   * Create and attach an FPS counter to the DOM
   * @param {HTMLElement} parent - Parent element to attach to
   * @param {Object} options - Styling options
   * @returns {HTMLElement} - The created FPS element
   */
  attach(parent, options = {}) {
    const defaults = {
      position: "absolute",
      top: "10px",
      right: "10px",
      background: "rgba(0,0,0,0.5)",
      color: "#fff",
      padding: "5px 10px",
      borderRadius: "3px",
      fontFamily: "monospace",
      fontSize: "12px",
      zIndex: 1000,
    };

    const styles = { ...defaults, ...options };

    this.element = document.createElement("div");
    this.element.textContent = "FPS: --";

    // Apply styles
    Object.assign(this.element.style, styles);

    // Append to parent
    parent.appendChild(this.element);

    return this.element;
  }

  /**
   * Update the frame counter - call this each frame
   */
  update() {
    this.frames++;

    const now = performance.now();
    const elapsed = now - this.lastUpdate;

    if (elapsed >= this.updateInterval) {
      this.fps = Math.round((this.frames * 1000) / elapsed);
      this.lastUpdate = now;
      this.frames = 0;

      if (this.element) {
        this.element.textContent = `FPS: ${this.fps}`;
      }
    }

    return this.fps;
  }
}

export default FpsCounter;
