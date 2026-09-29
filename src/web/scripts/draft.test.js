import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";

const template = fs.readFileSync(
  new URL("./draft.js", import.meta.url),
  "utf8",
);

const source = template
  .replaceAll(
    "__MADO_COMPOSER_SELECTOR__",
    JSON.stringify('#prompt-textarea,textarea[data-testid="prompt-textarea"],[contenteditable="true"][data-testid="prompt-textarea"]'),
  )
  .replaceAll("__MADO_MAX_DRAFT_CHARACTERS__", "200000");

class Element {
  constructor() {
    this.isConnected = true;
    this.childNodes = [];
    this.tagName = "TEXTAREA";
  }

  closest() {
    return this;
  }

  focus() {
    this.focused = true;
  }

  dispatchEvent(event) {
    this.onDispatch?.(event);
    return true;
  }
}

class HTMLTextAreaElement extends Element {
  constructor(value = "") {
    super();
    this.value = value;
  }
}

class HTMLInputElement extends Element {}

class Event {
  constructor(type, options = {}) {
    this.type = type;
    Object.assign(this, options);
  }
}

class InputEvent extends Event {}

class MutationObserver {
  observe() {}
  disconnect() {}
}

const Node = { TEXT_NODE: 3 };

function createPage({
  path = "/c/example",
  initialValue = "",
  observersAllowed = true,
} = {}) {
  const messages = [];
  const documentListeners = new Map();
  const windowListeners = new Map();
  let submitCount = 0;

  const composer = new HTMLTextAreaElement(initialValue);

  const addListener = (table, type, callback) => {
    const callbacks = table.get(type) ?? [];
    callbacks.push(callback);
    table.set(type, callbacks);
  };

  const dispatchTo = (table, type, event) => {
    for (const callback of table.get(type) ?? []) {
      callback(event);
    }
  };

  const sandbox = {
    console,
    setTimeout,
    clearTimeout,
    Element,
    HTMLTextAreaElement,
    HTMLInputElement,
    Event,
    InputEvent,
    MutationObserver,
    Node,
    location: { pathname: path },
    document: {
      hidden: false,
      documentElement: {},
      getElementById(id) {
        return id === "prompt-textarea" ? composer : null;
      },
      querySelector() {
        return composer;
      },
      addEventListener(type, callback) {
        addListener(documentListeners, type, callback);
      },
      execCommand() {
        return false;
      },
    },
    __madoBridge: {
      observersAllowed() {
        return observersAllowed;
      },
      post(type, payload) {
        messages.push({ type, payload });
        return true;
      },
    },
  };

  sandbox.window = sandbox;
  sandbox.globalThis = sandbox;
  sandbox.addEventListener = (type, callback) => {
    addListener(windowListeners, type, callback);
  };

  composer.onDispatch = (event) => {
    dispatchTo(documentListeners, event.type, {
      ...event,
      target: composer,
    });
    if (event.type === "submit") {
      submitCount += 1;
    }
  };

  vm.runInNewContext(source, sandbox);

  return {
    sandbox,
    composer,
    messages,
    emitDocument(type, event = {}) {
      dispatchTo(documentListeners, type, {
        target: composer,
        ...event,
      });
    },
    emitWindow(type, event = {}) {
      dispatchTo(windowListeners, type, event);
    },
    get submitCount() {
      return submitCount;
    },
  };
}

{
  const page = createPage();
  assert.equal(page.messages[0].type, "draft_ready");
  assert.deepEqual(JSON.parse(JSON.stringify(page.messages[0].payload)), {
    path: "/c/example",
    composer_empty: true,
  });

  page.composer.value = "debounced local draft";
  page.emitDocument("input");
  await new Promise((resolve) => setTimeout(resolve, 425));

  const saved = page.messages.find((message) => message.type === "draft_changed");
  assert.deepEqual(JSON.parse(JSON.stringify(saved?.payload)), {
    path: "/c/example",
    text: "debounced local draft",
  });
}

{
  const page = createPage({ observersAllowed: false });
  assert.equal(page.sandbox.__madoDraft, undefined);
  assert.equal(page.messages.length, 0);
}

{
  const page = createPage();
  const restored = page.sandbox.__madoDraft.restore("saved text", "/c/example");
  assert.equal(restored, true);
  assert.equal(page.composer.value, "saved text");
  assert.equal(page.submitCount, 0);

  const result = page.messages.findLast(
    (message) => message.type === "draft_restore_result",
  );
  assert.deepEqual(JSON.parse(JSON.stringify(result?.payload)), {
    path: "/c/example",
    restored: true,
    reason: "restored",
  });
}

{
  const page = createPage({ initialValue: "keep current text" });
  const restored = page.sandbox.__madoDraft.restore("saved text", "/c/example");
  assert.equal(restored, false);
  assert.equal(page.composer.value, "keep current text");

  const result = page.messages.findLast(
    (message) => message.type === "draft_restore_result",
  );
  assert.deepEqual(JSON.parse(JSON.stringify(result?.payload)), {
    path: "/c/example",
    restored: false,
    reason: "composer_not_empty",
  });
}

{
  const page = createPage();
  const restored = page.sandbox.__madoDraft.restore("saved text", "/c/other");
  assert.equal(restored, false);
  assert.equal(page.composer.value, "");

  const result = page.messages.findLast(
    (message) => message.type === "draft_restore_result",
  );
  assert.deepEqual(JSON.parse(JSON.stringify(result?.payload)), {
    path: "/c/other",
    restored: false,
    reason: "page_changed",
  });
}

console.log("draft.js tests passed");
