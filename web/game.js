const status = document.querySelector("#status");

try {
  const response = await fetch("./longsnake.wasm");
  if (!response.ok) throw new Error(`Game download failed (${response.status})`);
  const { instance } = await WebAssembly.instantiate(await response.arrayBuffer());
  const game = instance.exports;
  const canvas = document.querySelector("canvas");
  canvas.width = game.width();
  canvas.height = game.height();
  const context = canvas.getContext("2d");
  const image = context.createImageData(canvas.width, canvas.height);
  const keys = new Map([
    ["ArrowRight", 0], ["KeyD", 0],
    ["ArrowUp", 1], ["KeyW", 1],
    ["ArrowLeft", 2], ["KeyA", 2],
    ["ArrowDown", 3], ["KeyS", 3],
  ]);
  window.addEventListener("keydown", event => {
    const direction = keys.get(event.code);
    if (direction === undefined || event.ctrlKey || event.metaKey || event.altKey) return;
    event.preventDefault();
    if (!event.repeat) game.direction(direction);
  });
  let previous;
  function animate(now) {
    const pointer = game.frame(previous === undefined ? 0 : now - previous);
    previous = now;
    const pixels = new Uint32Array(game.memory.buffer, pointer, canvas.width * canvas.height);
    for (let i = 0; i < pixels.length; i++) {
      image.data[i * 4] = pixels[i] >>> 16 & 255;
      image.data[i * 4 + 1] = pixels[i] >>> 8 & 255;
      image.data[i * 4 + 2] = pixels[i] & 255;
      image.data[i * 4 + 3] = 255;
    }
    context.putImageData(image, 0, 0);
    requestAnimationFrame(animate);
  }
  document.addEventListener("visibilitychange", () => {
    previous = undefined;
  });
  status.textContent = "Ready";
  requestAnimationFrame(animate);
} catch (error) {
  status.textContent = `Unable to start: ${error.message}`;
}
