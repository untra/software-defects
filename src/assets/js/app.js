/* ============================================================
   Software Defects — interaction layer
   tabs · copy · lottie · reduced-motion · mobile nav · prefs
   ============================================================ */
(function () {
  "use strict";

  var reduceMotion = window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  /* ---- preferred code language, remembered across pages ---- */
  var PREF_KEY = "sd_lang_pref";
  function getPref() {
    try { return localStorage.getItem(PREF_KEY); } catch (e) { return null; }
  }
  function setPref(slug) {
    try { localStorage.setItem(PREF_KEY, slug); } catch (e) {}
  }

  function activate(viewer, slug) {
    var tabs = viewer.querySelectorAll(".lang-tab");
    var matched = false;
    tabs.forEach(function (tab) {
      var on = tab.getAttribute("data-lang") === slug;
      if (on) matched = true;
    });
    if (!matched) return false;
    tabs.forEach(function (tab) {
      var on = tab.getAttribute("data-lang") === slug;
      tab.classList.toggle("is-active", on);
      tab.setAttribute("aria-selected", on ? "true" : "false");
      var panel = viewer.querySelector("#" + tab.getAttribute("data-target"));
      if (panel) panel.classList.toggle("is-active", on);
    });
    return true;
  }

  /* JavaScript and TypeScript stand in for each other when a viewer lacks one */
  var LANG_ALIASES = { javascript: "typescript", typescript: "javascript" };

  function initViewers() {
    var pref = getPref();
    document.querySelectorAll(".code-viewer").forEach(function (viewer) {
      // apply remembered language if this viewer has it
      if (pref && !activate(viewer, pref) && LANG_ALIASES[pref]) activate(viewer, LANG_ALIASES[pref]);
      viewer.querySelectorAll(".lang-tab").forEach(function (tab) {
        tab.addEventListener("click", function () {
          var slug = tab.getAttribute("data-lang");
          // switch every viewer on the page so they stay in sync
          document.querySelectorAll(".code-viewer").forEach(function (v) {
            if (!activate(v, slug) && LANG_ALIASES[slug]) activate(v, LANG_ALIASES[slug]);
          });
          setPref(slug);
        });
        tab.addEventListener("keydown", function (e) {
          if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
          e.preventDefault();
          var all = Array.prototype.slice.call(viewer.querySelectorAll(".lang-tab"));
          var i = all.indexOf(tab);
          var next = e.key === "ArrowRight"
            ? all[(i + 1) % all.length]
            : all[(i - 1 + all.length) % all.length];
          next.focus();
          next.click();
        });
      });
    });
  }

  /* ---- copy to clipboard ---- */
  function initCopy() {
    document.querySelectorAll(".copy-btn").forEach(function (btn) {
      btn.addEventListener("click", function () {
        var panel = btn.closest(".corpus-code, .code-panel");
        var code = panel ? panel.querySelector("code") : null;
        if (!code) return;
        var text = code.innerText;
        var done = function () {
          var prev = btn.textContent;
          btn.textContent = btn.getAttribute("data-copied") || "Copied";
          btn.classList.add("is-copied");
          setTimeout(function () {
            btn.textContent = prev;
            btn.classList.remove("is-copied");
          }, 1400);
        };
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(text).then(done, done);
        } else {
          var ta = document.createElement("textarea");
          ta.value = text; document.body.appendChild(ta); ta.select();
          try { document.execCommand("copy"); } catch (e) {}
          document.body.removeChild(ta); done();
        }
      });
    });
  }

  /* ---- lottie glyphs (loaded only when on-screen) ---- */
  function playLottie(el) {
    if (el.dataset.loaded === "1") return;
    el.dataset.loaded = "1";
    if (typeof lottie === "undefined") return;
    var anim = lottie.loadAnimation({
      container: el,
      renderer: "svg",
      loop: true,
      autoplay: !reduceMotion,
      path: el.getAttribute("data-src")
    });
    if (reduceMotion) {
      // hold a representative mid-loop frame instead of animating
      anim.addEventListener("DOMLoaded", function () {
        anim.goToAndStop(Math.floor(anim.totalFrames * 0.5), true);
      });
    }
    el.__anim = anim;
  }

  function initLottie() {
    var glyphs = document.querySelectorAll(".lottie-glyph[data-src]");
    if (!glyphs.length) return;
    if (!("IntersectionObserver" in window)) {
      glyphs.forEach(playLottie); return;
    }
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (e.isIntersecting) { playLottie(e.target); io.unobserve(e.target); }
      });
    }, { rootMargin: "120px" });
    glyphs.forEach(function (g) { io.observe(g); });
  }

  /* ---- mobile nav ---- */
  function initNav() {
    var toggle = document.querySelector(".nav-toggle");
    var links = document.querySelector(".nav-links");
    if (!toggle || !links) return;
    toggle.addEventListener("click", function () {
      var open = links.classList.toggle("is-open");
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
    });
  }

  /* ---- language switcher ---- */
  function initLangSwitch() {
    var sel = document.querySelector(".lang-select");
    if (sel) sel.addEventListener("change", function () {
      window.location = sel.value;
    });
  }

  function ready(fn) {
    if (document.readyState !== "loading") fn();
    else document.addEventListener("DOMContentLoaded", fn);
  }
  ready(function () {
    initViewers();
    initCopy();
    initLottie();
    initNav();
    initLangSwitch();
  });
})();
