# Upstream Swift behavioral reference

This directory contains a deliberately small, read-only subset of the original GravityPoet/chatgpt-web-desktop Swift implementation.

It is **not** a macOS build target and does not contain packaging, signing, Sparkle, SwiftPM, or release infrastructure.

The retained files exist only because they are still materially useful for the remaining P2 parity work:

- **#17 completion notifications**
  - ChatGPTSwiftWeb/BrowserCompletionObserver.swift
  - ChatGPTSwiftWeb/CompletionNotificationService.swift
  - ChatGPTSwiftWeb/BrowserWindowController.swift
- **#18 profiles and settings**
  - ChatGPTSwiftWeb/AppDelegate.swift
  - ChatGPTSwiftWeb/AppSettingsWindowController.swift
  - ChatGPTSwiftWeb/BrowserDataBoundary.swift
  - ChatGPTSwiftWeb/BrowserSupport.swift
  - ChatGPTSwiftWeb/ConsentPreferenceSettings.swift
  - ChatGPTSwiftWeb/ProfileDataStoreInventory.swift
- **#19 diagnostics and privacy**
  - ChatGPTSwiftWeb/DiagnosticsWindowController.swift
  - ChatGPTSwiftWebCore/DiagnosticRedactor.swift
  - shared files above where integration context is required

Reimplement behavior idiomatically for Linux. Do not mechanically translate Swift or introduce macOS-only mechanisms into Mado.

Files removed from the working tree remain available through Git history if historical investigation is ever necessary.
