(() => {
  "use strict";

  const VERSION = __MADO_BRIDGE_VERSION__;
  const MAX_MESSAGE_BYTES = __MADO_MAX_MESSAGE_BYTES__;
  const CLOUDFLARE_CHALLENGE_SELECTOR = __MADO_CLOUDFLARE_SELECTOR__;
  const handler = globalThis.webkit?.messageHandlers?.mado;

  if (!handler || globalThis.__madoBridge?.version === VERSION) {
    return;
  }

  const trustedHost = () => {
    if (location.protocol !== "https:") {
      return false;
    }

    const host = location.hostname.toLowerCase();
    return host === "chatgpt.com"
      || host.endsWith(".chatgpt.com")
      || host === "openai.com"
      || host.endsWith(".openai.com");
  };

  const challengeActive = () =>
    location.pathname.startsWith("/cdn-cgi/")
      || document.querySelector(CLOUDFLARE_CHALLENGE_SELECTOR) !== null;

  const authLikePage = () => {
    const host = location.hostname.toLowerCase();
    const chatHost = host === "chatgpt.com" || host.endsWith(".chatgpt.com");

    if (!chatHost) {
      return true;
    }

    return /^\/(?:auth|login|signin|oauth)(?:\/|$)/i.test(location.pathname);
  };

  const pageKind = () => {
    if (!trustedHost()) {
      return "blocked";
    }
    if (challengeActive()) {
      return "challenge";
    }
    if (authLikePage()) {
      return "auth";
    }
    return "app";
  };

  const observersAllowed = () => pageKind() === "app";

  const post = (type, payload) => {
    let body;
    try {
      body = JSON.stringify({ v: VERSION, type, payload });
    } catch (_) {
      return false;
    }

    if (new TextEncoder().encode(body).byteLength > MAX_MESSAGE_BYTES) {
      return false;
    }

    try {
      handler.postMessage(body);
      return true;
    } catch (_) {
      return false;
    }
  };

  Object.defineProperty(globalThis, "__madoBridge", {
    configurable: false,
    enumerable: false,
    writable: false,
    value: Object.freeze({
      version: VERSION,
      pageKind,
      observersAllowed,
      post,
    }),
  });

  post("bridge_ready", {
    page: pageKind(),
    observers_allowed: observersAllowed(),
  });
})();
