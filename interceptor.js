(function () {
  // This whole script is only for Tauri...
  if (!window.__TAURI__) {
    return;
  }
  const proxySend = async function (body) {
    Object.defineProperty(this, "readyState", {writable: true});
    Object.defineProperty(this, "status", {writable: true});
    Object.defineProperty(this, "statusText", {writable: true});
    Object.defineProperty(this, "response", {writable: true});

    // TODO: Implement specs?
    //  https://fetch.spec.whatwg.org/#concept-bodyinit-extract
    //  https://xhr.spec.whatwg.org/#the-send()-method
    //  https://xhr.spec.whatwg.org/#interface-xmlhttprequest
    // TODO: Implement for all body types, this won't work with even FormData...
    // TODO: Check against content-type, do encodeURI, set header etc.
    const args = {
      request: {
        method: this.__hl_method,
        path: this.__hl_final_path,
        headers: this.__hl_headers || {},
        body: body,
      }
    };
    // TODO: Handover to a global object instead of using tauri_internals.
    // TODO: Handle Promise error, if command returns Err, propagate that...
    const response = await window.__TAURI_INTERNALS__.invoke("proxy_request", args);
    // Add lower-case versions of the headers, as per the latest spec.
    Object.entries(response.headers).map(([header, value]) => response.headers[header.toLowerCase()] = value);

    // TODO: abort support
    this.response = response.response;
    this.readyState = XMLHttpRequest.DONE;
    this.status = response.status;
    this.statusText = "";
    this.getAllResponseHeaders = () => Object.entries(response.headers).map(([header, value]) => `${header}: ${value}\r\n`).join("");
    this.getResponseHeader = (name) => response.headers[name.toLowerCase()] || null;

    this.dispatchEvent(new ProgressEvent("load"));
  }

// Patch Htmx calls to use Tauri proxy.
  window.addEventListener("DOMContentLoaded", () => {
    document.body.addEventListener("htmx:configRequest", (event) => {
      const internal_data = event.detail.elt["htmx-internal-data"];
      const xhr = internal_data.xhr;

      // Collect headers in the same way as htmx (removes ones with null value too).
      const headers = event.detail.headers;
      for (const header in headers)
        if (headers[header] == null)
          delete headers[header];

      const orig_open = xhr.open;
      xhr.send = proxySend;
      xhr.open = function () {
        this.__hl_method = arguments[0];
        this.__hl_final_path = arguments[1];
        this.__hl_path = internal_data.path;
        // Note: we ignore hx-request="noHeader", and skip uri encoding of values.
        this.__hl_headers = headers;
        orig_open.apply(this, arguments);
      }
    });
  });
}());
