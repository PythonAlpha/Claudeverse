// Profile search.
// JavaScript handles the page (typing, showing, hiding).
// Rust (WebAssembly) does the scoring. If the .wasm file isn't built yet,
// a small JavaScript scorer with the same rules takes over.

(() => {
  const input = document.getElementById("q");
  const status = document.getElementById("search-status");
  const ITEMS =
    "#about p, #people p, .grid > div, .top li, .flow li, .skills li, .skills-fallback li";

  // ---- JavaScript fallback scorer (same rules as the Rust one) -----------
  const jsScore = (query, text) => {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (words.length === 0) return 1;
    const hay = text.toLowerCase();
    let total = 0;
    for (const w of words) {
      const pos = hay.indexOf(w);
      if (pos === -1) return 0;
      total += 10;
      if (pos === 0 || /\s/.test(hay[pos - 1])) total += 5;
    }
    return total;
  };

  let scorer = jsScore;
  let engine = "JavaScript";

  // ---- Load the Rust scorer ---------------------------------------------
  const loadRust = async () => {
    const response = await fetch("profile_search.wasm");
    if (!response.ok) throw new Error("wasm not found");
    const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), {});
    const x = instance.exports;
    const cap = x.capacity();
    const enc = new TextEncoder();

    // Write text into a buffer inside the WASM module, truncating safely.
    const write = (ptr, str) => {
      const bytes = enc.encode(str).subarray(0, cap);
      new Uint8Array(x.memory.buffer, ptr, cap).set(bytes);
      return bytes.length;
    };

    scorer = (query, text) => {
      const qlen = write(x.query_ptr(), query);
      const tlen = write(x.text_ptr(), text);
      return x.score(qlen, tlen);
    };
    engine = "Rust";
  };

  // ---- Page behaviour ---------------------------------------------------
  const run = () => {
    const query = input.value.trim();
    let shown = 0;
    let total = 0;

    document.querySelectorAll(ITEMS).forEach((el) => {
      total += 1;
      const match = scorer(query, el.textContent.replace(/\s+/g, " ").trim()) > 0;
      el.hidden = !match;
      if (match) shown += 1;
    });

    // Hide whole sections that have nothing left to show.
    document.querySelectorAll("main section").forEach((sec) => {
      const items = sec.querySelectorAll(ITEMS);
      const empty = items.length > 0 && [...items].every((el) => el.hidden);
      sec.hidden = empty;
    });

    if (!query) {
      status.textContent = `Searching with ${engine}.`;
    } else if (shown === 0) {
      status.textContent = "No matches. Try a shorter word, like “code” or “music”.";
    } else {
      status.textContent = `${shown} of ${total} matches · scored by ${engine}.`;
    }
  };

  input.addEventListener("input", run);

  // The Ruby code fills in the skills list a moment after load, so wait for it.
  const start = async () => {
    try {
      await loadRust();
    } catch (e) {
      console.info("Using the JavaScript scorer. Build the Rust one with rust/build.sh.", e);
    }
    run();
  };

  window.addEventListener("load", () => setTimeout(start, 400));
})();
