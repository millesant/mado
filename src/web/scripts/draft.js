(() => {
  "use strict";

  const bridge = globalThis.__madoBridge;
  const COMPOSER_SELECTOR = __MADO_COMPOSER_SELECTOR__;
  const MAX_DRAFT_CHARACTERS = __MADO_MAX_DRAFT_CHARACTERS__;
  const SAVE_DEBOUNCE_MS = 350;

  if (!bridge || globalThis.__madoDraft || !bridge.observersAllowed()) {
    return;
  }

  const validPath = (path) =>
    typeof path === "string"
      && path.length <= 512
      && /^\/[A-Za-z0-9_/-]*$/.test(path)
      && !path.startsWith("//");

  const textOf = (element) => {
    if (!element) {
      return "";
    }
    if (element instanceof HTMLTextAreaElement || element instanceof HTMLInputElement) {
      return element.value || "";
    }

    const read = (node) => {
      if (node.nodeType === Node.TEXT_NODE) {
        return node.nodeValue || "";
      }
      if (!(node instanceof Element)) {
        return "";
      }
      if (node.tagName === "BR") {
        return "\n";
      }

      let result = Array.from(node.childNodes, read).join("");
      if (/^(P|DIV|LI|PRE)$/.test(node.tagName) && node !== element) {
        result += "\n";
      }
      return result;
    };

    return read(element).replace(/\n$/, "");
  };

  let composer = null;
  let pending = null;
  let saveTimer = 0;
  let composing = false;
  let bootstrapObserver = null;
  let attachScheduled = false;

  const post = (type, payload) => {
    if (!bridge.observersAllowed()) {
      return false;
    }
    return bridge.post(type, payload);
  };

  const announceReady = () => {
    if (!composer?.isConnected || !validPath(location.pathname)) {
      return;
    }

    post("draft_ready", {
      path: location.pathname,
      composer_empty: textOf(composer).trim().length === 0,
    });
  };

  const attach = () => {
    attachScheduled = false;
    if (!bridge.observersAllowed()) {
      return;
    }

    const next = document.getElementById("prompt-textarea")
      || document.querySelector(COMPOSER_SELECTOR);
    if (!next) {
      return;
    }

    if (next !== composer) {
      composer = next;
      bootstrapObserver?.disconnect();
    }
    announceReady();
  };

  const scheduleAttach = () => {
    if (attachScheduled) {
      return;
    }
    attachScheduled = true;
    setTimeout(attach, 150);
  };

  const flush = () => {
    clearTimeout(saveTimer);
    saveTimer = 0;

    if (!bridge.observersAllowed() || composing || !pending) {
      return;
    }

    const { element, path } = pending;
    pending = null;
    if (!validPath(path)) {
      return;
    }

    const text = textOf(element);
    if (!text.trim() || text.length > MAX_DRAFT_CHARACTERS) {
      return;
    }

    post("draft_changed", { path, text });
  };

  const onInput = (event) => {
    if (!bridge.observersAllowed() || composing) {
      return;
    }

    const target = event.target instanceof Element
      ? event.target.closest(COMPOSER_SELECTOR)
      : null;
    if (!target || !validPath(location.pathname)) {
      return;
    }

    composer = target;
    pending = { element: target, path: location.pathname };
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flush, SAVE_DEBOUNCE_MS);
  };

  const reportRestore = (path, restored, reason) => {
    bridge.post("draft_restore_result", {
      path,
      restored,
      reason,
    });
  };

  const restore = (text, path) => {
    if (!bridge.observersAllowed()) {
      reportRestore(path, false, "blocked");
      return false;
    }
    if (!validPath(path) || location.pathname !== path) {
      reportRestore(path, false, "page_changed");
      return false;
    }
    if (typeof text !== "string" || !text.trim() || text.length > MAX_DRAFT_CHARACTERS) {
      reportRestore(path, false, "invalid_draft");
      return false;
    }

    attach();
    if (!composer?.isConnected) {
      reportRestore(path, false, "composer_missing");
      return false;
    }

    if (textOf(composer).trim().length > 0) {
      reportRestore(path, false, "composer_not_empty");
      return false;
    }

    composer.focus();
    if (composer instanceof HTMLTextAreaElement || composer instanceof HTMLInputElement) {
      const prototype = composer instanceof HTMLTextAreaElement
        ? HTMLTextAreaElement.prototype
        : HTMLInputElement.prototype;
      const descriptor = Object.getOwnPropertyDescriptor(prototype, "value");
      if (descriptor?.set) {
        descriptor.set.call(composer, text);
      } else {
        composer.value = text;
      }
    } else {
      const inserted = document.execCommand("insertText", false, text);
      if (!inserted && !textOf(composer).trim()) {
        composer.textContent = text;
      }
    }

    composer.dispatchEvent(new InputEvent("input", {
      bubbles: true,
      inputType: "insertText",
      data: text,
    }));
    composer.dispatchEvent(new Event("change", { bubbles: true }));

    reportRestore(path, true, "restored");
    return true;
  };

  document.addEventListener("input", onInput, true);
  document.addEventListener("change", onInput, true);
  document.addEventListener("compositionstart", (event) => {
    if (event.target instanceof Element && event.target.closest(COMPOSER_SELECTOR)) {
      composing = true;
    }
  }, true);
  document.addEventListener("compositionend", (event) => {
    composing = false;
    onInput(event);
  }, true);
  document.addEventListener("submit", flush, true);
  window.addEventListener("pagehide", flush);
  window.addEventListener("popstate", () => {
    flush();
    scheduleAttach();
  });
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      flush();
    }
  });

  Object.defineProperty(globalThis, "__madoDraft", {
    configurable: false,
    enumerable: false,
    writable: false,
    value: Object.freeze({ restore, flush }),
  });

  attach();
  if (!composer) {
    bootstrapObserver = new MutationObserver(scheduleAttach);
    bootstrapObserver.observe(document.documentElement, {
      childList: true,
      subtree: true,
    });
    setTimeout(() => bootstrapObserver?.disconnect(), 10_000);
  }
})();
