# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class dev.webrogue.launcher.* {
  native <methods>;
}

-keep class dev.webrogue.launcher.WryActivity {
  public <init>(...);

  void setWebView(dev.webrogue.launcher.RustWebView);
  java.lang.Class getAppClass(...);
  java.lang.String getVersion();
}

-keep class dev.webrogue.launcher.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class dev.webrogue.launcher.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void setAutoPlay(...);
  void setUserAgent(...);
  void evalScript(...);
}

-keep class dev.webrogue.launcher.RustWebChromeClient,dev.webrogue.launcher.RustWebViewClient {
  public <init>(...);
}