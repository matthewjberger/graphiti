import init from "./engine.js";

const buffered = [];
self.onmessage = (event) => buffered.push(event);

init().then(() => {
  for (const event of buffered) {
    self.onmessage(event);
  }
});
