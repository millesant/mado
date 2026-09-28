import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder } from "node:util";

const template = fs.readFileSync(
  new URL("./bootstrap.js", import.meta.url),
  "utf8",
);

const source = template
  .replaceAll("__MADO_BRIDGE_VERSION__", "1")
  .replaceAll("__MADO_MAX_MESSAGE_BYTES__", String(16 * 1024))
  .replaceAll("__MADO_CLOUDFLARE_SELECTOR__", JSON.stringify("[data-test-challenge]"));

function loadBridge({
  host = "chatgpt.com",
  pathname = "/",
  challenge = false,
} = {}) {
  const messages = [];
  const context = {
    TextEncoder,
    location: {
      protocol: "https:",
      hostname: host,
      pathname,
    },
    document: {
      querySelector() {
        return challenge ? {} : null;
      },
    },
    webkit: {
      messageHandlers: {
        mado: {
          postMessage(message) {
            messages.push(message);
          },
        },
      },
    },
  };

  vm.runInNewContext(source, context);

  return {
    bridge: context.__madoBridge,
    messages: messages.map((message) => JSON.parse(message)),
  };
}

{
  const { bridge, messages } = loadBridge();
  assert.equal(bridge.version, 1);
  assert.equal(bridge.pageKind(), "app");
  assert.equal(bridge.observersAllowed(), true);
  assert.deepEqual(messages[0], {
    v: 1,
    type: "bridge_ready",
    payload: {
      page: "app",
      observers_allowed: true,
    },
  });
}

{
  const { bridge } = loadBridge({ host: "auth.openai.com" });
  assert.equal(bridge.pageKind(), "auth");
  assert.equal(bridge.observersAllowed(), false);
}

{
  const { bridge } = loadBridge({ pathname: "/auth/login" });
  assert.equal(bridge.pageKind(), "auth");
  assert.equal(bridge.observersAllowed(), false);
}

{
  const { bridge } = loadBridge({ challenge: true });
  assert.equal(bridge.pageKind(), "challenge");
  assert.equal(bridge.observersAllowed(), false);
}

{
  const { bridge, messages } = loadBridge();
  const posted = bridge.post("oversized_test", {
    text: "x".repeat(20 * 1024),
  });
  assert.equal(posted, false);
  assert.equal(messages.length, 1);
}

console.log("bootstrap.js tests passed");
