#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Programmatic Lottie generator for the Bugs of Mutated State sub-defects.

Emits small (240x240) Lottie JSON files built from *text layers* (ty:5), so each
animation spells out the defect's name in motion — echoing the typographic
treatments from the "Bugs of Mutated State" deck. lottie-web renders text layers
using the fonts named in the `fonts` list, which the site already loads from
Google Fonts (JetBrains Mono + Space Grotesk), so no embedded glyph data is
needed.

Each family carries the deck's accent: green mutated-state (#78A65A), red
shared-state (#DD3333), and blue unstructured-state (#3D85C6). `write_all(outdir)`
writes one <slug>.json per treatment plus the regenerated family glyphs. The build
calls this instead of copying static JSON from disk.
"""
import json
import os

import i18n

# ── palette (hex → 0..1 rgb) ────────────────────────────────────────────────
GREEN  = [0.4706, 0.6510, 0.3529]   # #78A65A  primary
BRIGHT = [0.5765, 0.7686, 0.4902]   # #93C47D  soft / highlight
DEEP   = [0.3098, 0.4745, 0.2588]   # #4F7942  deep
LIGHT  = [0.9059, 0.9216, 0.9412]   # #E7EBF0  on-dark text
DIM    = [0.4196, 0.4824, 0.4353]   # muted green-grey for kickers
WARN   = [0.9647, 0.6235, 0.2667]   # warm accent for "dropped"/error char

# ── shared-state palette (red) — #DD3333 / #F87171 / #B91C1C ────────────────
RED      = [0.8667, 0.2000, 0.2000]   # #DD3333  primary
RED_SOFT = [0.9725, 0.4431, 0.4431]   # #F87171  soft / highlight
RED_DEEP = [0.7255, 0.1098, 0.1098]   # #B91C1C  deep / dead / dropped

# ── unstructured-state palette (blue) — #3D85C6 / #6FA8DC / #0B5394 ──────────
BLUE      = [0.2392, 0.5216, 0.7765]   # #3D85C6  primary (the deck's blue)
BLUE_SOFT = [0.4353, 0.6588, 0.8627]   # #6FA8DC  soft / highlight
BLUE_DEEP = [0.0431, 0.3255, 0.5804]   # #0B5394  deep

# ── common-defects palette (gray) — #94A3B8 / #CBD5E1 / #64748B ──────────────
GRAY      = [0.5804, 0.6392, 0.7216]   # #94A3B8  primary (slate)
GRAY_SOFT = [0.7961, 0.8353, 0.8824]   # #CBD5E1  soft / highlight
GRAY_DEEP = [0.3922, 0.4549, 0.5451]   # #64748B  deep / kicker

# ── compound-defects child colors (the three emergent classes) ──────────────
ND_YEL = [1.0000, 0.8980, 0.6000]      # #FFE599  nondeterminism (yellow)
DC_MAG = [0.7608, 0.4824, 0.6275]      # #C27BA0  data-corruption (magenta)
RT_CYN = [0.6235, 0.7725, 0.9098]      # #9FC5E8  runtime-errors (cyan)

FONT_MONO = "JetBrainsMono-Bold"
FONT_MONO_REG = "JetBrainsMono-Regular"
FONT_MONO_ITAL = "JetBrainsMono-Italic"
FONT_SANS = "SpaceGrotesk-Bold"

# Scripts with letter case + true italics get the full case/italic treatment;
# others (CJK, Arabic) degrade gracefully to weight/scale/fade-only waves.
LATIN_LANGS = {"en", "es", "fr", "de", "pt-br"}

W = H = 240
FR = 30
OP = 90  # 3-second loop

EASE_O = {"x": [0.33], "y": [0.0]}
EASE_I = {"x": [0.67], "y": [1.0]}


# ── low-level builders ──────────────────────────────────────────────────────
def _prop(keys, hold=False, spatial=False):
    """Animated property. keys = [(frame, value), ...]; value scalar or list.

    Collapses duplicate/non-increasing frame times (keeping the last value at a
    given frame) so lottie always sees strictly increasing keyframe times.
    """
    merged = {}
    for t, v in keys:
        merged[int(round(t))] = v
    keys = [(t, merged[t]) for t in sorted(merged)]
    arr = []
    for i, (t, v) in enumerate(keys):
        val = list(v) if isinstance(v, (list, tuple)) else [v]
        k = {"t": t, "s": val}
        if hold:
            k["h"] = 1
        elif i < len(keys) - 1:
            k["o"] = EASE_O
            k["i"] = EASE_I
        arr.append(k)
    return {"a": 1, "k": arr}


def _static(v):
    return {"a": 0, "k": list(v) if isinstance(v, (list, tuple)) else v}


def _hashf(key, salt):
    """Deterministic pseudo-random value in [0, 1).

    Uses an explicit multiplicative hash (NOT Python's salted built-in hash() or
    random) so builds stay byte-reproducible while still looking scattered —
    matching the hand-coded deterministic-jitter style used elsewhere here.
    """
    return ((key * 2654435761 + salt * 40503) % 100003) / 100003.0


def _doc(text, size, color, justify=2, tracking=0, line=None, font=FONT_MONO):
    """A single static text document."""
    return {"d": {"k": [{"s": {
        "s": size, "f": font, "t": text, "j": justify, "tr": tracking,
        "lh": line if line is not None else round(size * 1.2, 1), "ls": 0,
        "fc": color,
    }, "t": 0}]}, "p": {}, "m": {"g": 1, "a": _static([0, 0])}, "a": []}


def _doc_styled(frames, size, justify=2, tracking=0, line=None):
    """Document keyframes where each frame carries its own text/font/color.

    frames = [(t, text, font, color), ...]; values are hold-swapped (snap), so
    this drives case+weight waves (lowercase regular -> UPPERCASE bold) and
    letter->digit morphs without needing glyph interpolation.
    """
    k = []
    last_t = None
    for t, text, font, color in frames:
        t = int(round(t))
        if last_t is not None and t <= last_t:
            t = last_t + 1  # keep frame times strictly increasing
        last_t = t
        k.append({"s": {
            "s": size, "f": font, "t": text, "j": justify, "tr": tracking,
            "lh": line if line is not None else round(size * 1.2, 1), "ls": 0,
            "fc": color,
        }, "t": t})
    return {"d": {"k": k}, "p": {}, "m": {"g": 1, "a": _static([0, 0])}, "a": []}


def _char_layout(word, size, advance_ratio=0.62, max_w=216):
    """Center `word` across the canvas, one slot per character (spaces kept).

    Returns [(char, x_center), ...]. Adapts the per-char advance so longer
    translated words still fit, since word length varies by language.
    """
    n = len(word)
    if n == 0:
        return []
    adv = size * advance_ratio
    if n * adv > max_w:
        adv = max_w / n
    total = n * adv
    x0 = W / 2 - total / 2 + adv / 2
    return [(word[i], x0 + i * adv) for i in range(n)]


def _doc_seq(frames, size, color, justify=2, tracking=0, line=None, font=FONT_MONO):
    """Document keyframes that swap the displayed text over time (hold)."""
    k = []
    last_t = None
    for t, text in frames:
        if last_t is not None and t <= last_t:
            t = last_t + 1  # keep frame times strictly increasing
        last_t = t
        k.append({"s": {
            "s": size, "f": font, "t": text, "j": justify, "tr": tracking,
            "lh": line if line is not None else round(size * 1.2, 1), "ls": 0,
            "fc": color,
        }, "t": t})
    return {"d": {"k": k}, "p": {}, "m": {"g": 1, "a": _static([0, 0])}, "a": []}


def _text(ind, doc, pos, *, opacity=None, position=None, scale=None,
          rotation=None, ip=0, op=OP, name="t"):
    ks = {
        "o": opacity if opacity is not None else _static(100),
        "r": rotation if rotation is not None else _static(0),
        "p": position if position is not None else _static([pos[0], pos[1], 0]),
        "a": _static([0, 0, 0]),
        "s": scale if scale is not None else _static([100, 100, 100]),
    }
    return {"ddd": 0, "ind": ind, "ty": 5, "nm": name, "sr": 1, "ks": ks,
            "t": doc, "ip": ip, "op": op, "st": 0, "bm": 0}


def _rect(ind, pos, size, color, *, opacity=None, position=None, scale=None,
          rotation=None, roundness=4, ip=0, op=OP, name="r"):
    """A filled rounded-rectangle shape layer (for carets / value boxes / bars)."""
    shape = {"ty": "gr", "it": [
        {"ty": "rc", "d": 1, "s": _static(size), "p": _static([0, 0]),
         "r": _static(roundness), "nm": "rc"},
        {"ty": "fl", "c": _static(color), "o": _static(100), "r": 1, "nm": "fl"},
        {"ty": "tr", "p": _static([0, 0]), "a": _static([0, 0]),
         "s": _static([100, 100]), "r": _static(0), "o": _static(100)},
    ], "nm": "grp"}
    ks = {
        "o": opacity if opacity is not None else _static(100),
        "r": rotation if rotation is not None else _static(0),
        "p": position if position is not None else _static([pos[0], pos[1], 0]),
        "a": _static([0, 0, 0]),
        "s": scale if scale is not None else _static([100, 100, 100]),
    }
    return {"ddd": 0, "ind": ind, "ty": 4, "nm": name, "sr": 1, "ks": ks,
            "shapes": [shape], "ip": ip, "op": op, "st": 0, "bm": 0}


def _comp(name, layers, op=OP):
    return {"v": "5.7.4", "fr": FR, "ip": 0, "op": op, "w": W, "h": H,
            "nm": name, "ddd": 0, "assets": [], "fonts": {"list": [
                {"fName": FONT_MONO, "fFamily": "JetBrains Mono",
                 "fStyle": "Bold", "fWeight": "700", "ascent": 75},
                {"fName": FONT_MONO_REG, "fFamily": "JetBrains Mono",
                 "fStyle": "Regular", "fWeight": "400", "ascent": 75},
                {"fName": FONT_MONO_ITAL, "fFamily": "JetBrains Mono",
                 "fStyle": "Italic", "fWeight": "400", "ascent": 75},
                {"fName": FONT_SANS, "fFamily": "Space Grotesk",
                 "fStyle": "Bold", "fWeight": "700", "ascent": 75},
            ]}, "layers": layers}


def _kicker(ind, text, y=58, color=DIM, size=13, tracking=160):
    return _text(ind, _doc(text, size, color, tracking=tracking), (W / 2, y), name="kicker")


def _blink(ind, x, y, color=BRIGHT, w=8, h=34):
    """A blinking block caret."""
    blink = _prop([(0, 100), (15, 100), (16, 0), (45, 0), (46, 100),
                   (75, 100), (76, 0), (90, 0)], hold=True)
    return _rect(ind, (x, y), [w, h], color, opacity=blink, roundness=1, name="caret")


# ── per-defect treatments ───────────────────────────────────────────────────
def cycle(words=("var", "let", "mut")):
    """var → let → mut cross-fading in place: a binding being reassigned."""
    layers = [_kicker(1, "REASSIGNED")]
    span = OP // len(words)
    fade = 7
    for i, w in enumerate(words):
        a = i * span
        b = a + span
        keys = [(0, 0)]
        if a > 0:
            keys.append((max(0, a - fade), 0))
        keys.append((a, 100))
        keys.append((b - fade, 100))
        keys.append((b, 0))
        keys.append((OP, 100 if i == 0 else 0))
        # de-dup / sort by frame
        seen, clean = {}, []
        for t, v in keys:
            seen[t] = v
        for t in sorted(seen):
            clean.append((t, seen[t]))
        layers.append(_text(10 + i, _doc(w, 58, BRIGHT, tracking=20),
                            (W / 2 - 16, 138), opacity=_prop(clean), name=w))
    layers.append(_blink(40, W / 2 + 58, 118))
    return _comp("var-let-mut", layers)


def truncate(word="error"):
    """"erro" + a trailing "r" that slides one step off and fades — off by one."""
    stem = word[:-1]
    tail = word[-1]
    layers = [_kicker(1, "OFF  BY  ONE")]
    # stem sits left-justified so the tail can sit exactly after it
    layers.append(_text(10, _doc(stem, 60, LIGHT, justify=0), (52, 138), name="stem"))
    # tail: drifts right + up and fades out, then snaps back (loops)
    pos = _prop([(0, [170, 138, 0]), (40, [210, 120, 0]), (44, [210, 120, 0]),
                 (45, [170, 138, 0]), (OP, [170, 138, 0])], spatial=True)
    op = _prop([(0, 100), (34, 100), (44, 0), (45, 0), (52, 100), (OP, 100)], hold=False)
    layers.append(_text(11, _doc(tail, 60, WARN, justify=0), (170, 138),
                        position=pos, opacity=op, name="tail"))
    return _comp("off-by-one-errors", layers)


def cascade(text="for loop control", n=5):
    """The phrase repeats, stacking downward — nested loops piling up."""
    layers = [_kicker(1, "for ( ; ; )")]
    top = 88
    step = 22
    per = 12
    for i in range(n):
        y = top + i * step
        a = i * per
        op = _prop([(0, 0), (max(0, a - 4), 0), (a, 100),
                    (OP - 10, 100), (OP, 0)], hold=False)
        col = BRIGHT if i % 2 == 0 else GREEN
        layers.append(_text(10 + i, _doc(text, 22, col, tracking=10),
                            (W / 2, y), opacity=op, name=f"row{i}"))
    return _comp("for-loop-control-flow", layers)


def slide():
    """`this` stays put; `mutation` drifts away and fades — escaping reference."""
    layers = [_kicker(1, "this  →")]
    layers.append(_text(10, _doc("this", 50, LIGHT), (W / 2, 118), name="this"))
    pos = _prop([(0, [W / 2, 160, 0]), (60, [W / 2 + 70, 160, 0]),
                 (66, [W / 2 + 70, 160, 0]), (67, [W / 2, 160, 0]),
                 (OP, [W / 2, 160, 0])], spatial=True)
    op = _prop([(0, 100), (50, 100), (64, 0), (67, 0), (74, 100), (OP, 100)])
    layers.append(_text(11, _doc("mutation", 30, GREEN, tracking=20), (W / 2, 160),
                        position=pos, opacity=op, name="mutation"))
    return _comp("this-mutation", layers)


def ticker():
    """time / money / entropy stacked; a counter ticks and the active row glows."""
    rows = ["time", "money", "entropy"]
    layers = [_kicker(1, "Δ  IMPURE  SOURCES")]
    span = OP // 3
    ys = [104, 138, 172]
    for i, r in enumerate(rows):
        a = i * span
        glow = _prop([(0, 35), (max(0, a - 4), 35), (a, 100),
                      (a + span - 6, 100), (a + span, 35), (OP, 35 if i else 100)],
                     hold=False)
        layers.append(_text(10 + i, _doc(r, 30, BRIGHT, justify=0), (44, ys[i]),
                            opacity=glow, name=r))
    # a counting number on the right edge
    nums = [(t, f"{(t * 137) % 1000:03d}") for t in range(0, OP, 6)]
    layers.append(_text(40, _doc_seq(nums, 26, DEEP, justify=1), (212, 138),
                        name="count"))
    return _comp("time-money-entropy", layers)


def corrupt():
    """PURE → IM slides in → IMPURE; FUNCTIONS sits below."""
    layers = [_kicker(1, "SIDE  EFFECTS")]
    # "IM" prefix slides in from the left and prepends
    pos = _prop([(0, [40, 122, 0]), (4, [40, 122, 0]), (22, [78, 122, 0]),
                 (64, [78, 122, 0]), (80, [40, 122, 0]), (OP, [40, 122, 0])],
                spatial=True)
    op = _prop([(0, 0), (4, 0), (20, 100), (66, 100), (80, 0), (OP, 0)])
    layers.append(_text(10, _doc("IM", 46, WARN, justify=1), (78, 122),
                        position=pos, opacity=op, name="im"))
    layers.append(_text(11, _doc("PURE", 46, LIGHT, justify=0), (96, 122), name="pure"))
    layers.append(_text(12, _doc("FUNCTIONS", 24, GREEN, tracking=40),
                        (W / 2, 168), name="functions"))
    return _comp("impure-functions", layers)


def expand():
    """One M expands into Macro · Mixin · Misuse — macro expansion."""
    layers = [_kicker(1, "#define")]
    words = ["Macro", "Mixin", "Misuse"]
    ys = [104, 138, 172]
    for i, w in enumerate(words):
        a = 6 + i * 14
        sc = _prop([(0, [40, 40, 100]), (max(0, a - 6), [40, 40, 100]),
                    (a + 8, [112, 112, 100]), (a + 14, [100, 100, 100]),
                    (OP - 10, [100, 100, 100]), (OP, [40, 40, 100])], spatial=True)
        op = _prop([(0, 0), (max(0, a - 6), 0), (a + 6, 100),
                    (OP - 10, 100), (OP, 0)])
        col = [BRIGHT, GREEN, DEEP][i]
        layers.append(_text(10 + i, _doc(w, 30, col, tracking=8), (W / 2, ys[i]),
                            scale=sc, opacity=op, name=w))
    return _comp("macro-mixin-misuse", layers)


def counter():
    """A counter climbs to its int8 max, then one ++ wraps it to the minimum."""
    layers = [_kicker(1, "i8  ++")]
    # count up to 127 (int8 max), then a single bump wraps to -128
    seq = [124, 125, 126, 127, -128, -127, -126, -125]
    wrap_i = seq.index(-128)
    # the number flashes WARN on the wrap frame, LIGHT otherwise
    frames = []
    for j, val in enumerate(seq):
        t = round(j * OP / len(seq))
        col = WARN if j == wrap_i else LIGHT
        frames.append((t, str(val), FONT_MONO, col))
    layers.append(_text(10, _doc_styled(frames, 64, justify=2),
                        (W / 2, 150), name="num"))
    # caption: '++' climbing, 'MAX' at the top, 'wrap!' on rollover
    cap = _doc_styled([
        (0, "++", FONT_MONO, BRIGHT),
        (round((wrap_i - 1) * OP / len(seq)), "MAX", FONT_MONO, BRIGHT),
        (round(wrap_i * OP / len(seq)), "wrap!", FONT_MONO, WARN),
    ], 26, justify=0)
    layers.append(_text(11, cap, (150, 96), name="cap"))
    return _comp("increment-decrement-operators", layers)


def init_glitch():
    """A binding reads garbage '???' before it's assigned, then resolves to 0."""
    layers = [_kicker(1, "let x")]
    # the value box in the middle
    layers.append(_rect(5, (W / 2, 140), [78, 52], DEEP, opacity=_static(100),
                        roundness=8, name="box"))
    # the slot flickers uninitialized garbage, then snaps to a defined value
    val_frames = [
        (0, "?", FONT_MONO, WARN),
        (10, "█", FONT_MONO, WARN),
        (18, "?!", FONT_MONO, WARN),
        (28, "▓", FONT_MONO, WARN),
        (OP // 2, "0", FONT_MONO, LIGHT),
        (OP - 8, "0", FONT_MONO, LIGHT),
    ]
    layers.append(_text(6, _doc_styled(val_frames, 34, justify=2), (W / 2, 150), name="val"))
    # caption: 'undefined' while garbage, '= 0' once defined
    cap = _doc_styled([
        (0, "undefined", FONT_MONO, WARN),
        (OP // 2, "= 0", FONT_MONO, BRIGHT),
    ], 20, justify=2)
    layers.append(_text(10, cap, (W / 2, 198), name="cap"))
    return _comp("improper-initialization", layers)


def mutated_family_glyph(lang):
    """mutated-state family glyph: each character rises from lowercase to
    UPPERCASE + bold in a traveling wave (green). Non-Latin scripts keep the
    glyph and degrade to a weight/scale/brighten wave (no case change)."""
    latin = lang in LATIN_LANGS
    word = i18n.t(lang, "anim_mutated")
    if latin:
        word = word.lower()
    size = 46
    layout = _char_layout(word, size)
    y = 130
    rise, fall = 5, 6
    lo, hi = 6, OP - 26  # flip window; leaves room for the pop to finish in-loop
    layers, ind = [], 10
    for i, (ch, x) in enumerate(layout):
        if ch == " ":
            continue
        # scattered, word-specific flip — order no longer tracks position
        key = i * 7 + ord(ch)
        t0 = lo + _hashf(key, 11) * (hi - lo)
        hold = 5 + int(_hashf(key, 29) * 7)      # 5..11 frames
        pop = 116 + int(_hashf(key, 53) * 16)    # 116..131 % scale
        up = ch.upper() if latin else ch
        frames = [
            (0, ch, FONT_MONO_REG, GREEN),
            (t0, ch, FONT_MONO_REG, GREEN),
            (t0 + rise, up, FONT_MONO, BRIGHT),
            (t0 + rise + hold, up, FONT_MONO, BRIGHT),
            (t0 + rise + hold + fall, ch, FONT_MONO_REG, GREEN),
            (OP, ch, FONT_MONO_REG, GREEN),
        ]
        sc = _prop([(0, [100, 100, 100]), (t0, [100, 100, 100]),
                    (t0 + rise, [pop, pop, 100]),
                    (t0 + rise + hold, [pop, pop, 100]),
                    (t0 + rise + hold + fall, [100, 100, 100]),
                    (OP, [100, 100, 100])])
        layers.append(_text(ind, _doc_styled(frames, size), (x, y),
                            scale=sc, name="m%d" % i))
        ind += 1
    return _comp("mutated-state", layers)


# ── shared-state treatments (red, one per cause) ────────────────────────────
def race():
    """'race' / 'condition' pulse out of phase — two actors interleaving."""
    layers = [_kicker(1, "INTERLEAVING")]
    a_op = _prop([(0, 100), (20, 100), (30, 30), (60, 30), (70, 100), (OP, 100)])
    b_op = _prop([(0, 30), (20, 30), (30, 100), (60, 100), (70, 30), (OP, 30)])
    layers.append(_text(10, _doc("race", 52, RED_SOFT, tracking=10), (W / 2, 112),
                        opacity=a_op, name="race"))
    layers.append(_text(11, _doc("condition", 34, RED, tracking=8), (W / 2, 156),
                        opacity=b_op, name="condition"))
    layers.append(_blink(30, W / 2 + 74, 92, color=RED_DEEP))
    return _comp("race-conditions", layers)


def contention():
    """RESOURCE / CONTENTION stacked; the shared pool depletes and refills."""
    layers = [_kicker(1, "THE COMMONS")]
    layers.append(_text(10, _doc("RESOURCE", 30, LIGHT, tracking=20), (W / 2, 98), name="resource"))
    layers.append(_text(11, _doc("CONTENTION", 26, RED, tracking=14), (W / 2, 130), name="contention"))
    sc = _prop([(0, [100, 100, 100]), (42, [6, 100, 100]), (48, [6, 100, 100]),
                (58, [100, 100, 100]), (OP, [100, 100, 100])])
    layers.append(_rect(20, (W / 2, 170), [150, 14], RED_DEEP, scale=sc, roundness=3, name="pool"))
    return _comp("resource-contention", layers)


def stack_of():
    """'stack' rows pile upward until the top overflows off-frame."""
    layers = [_kicker(1, "no base case")]
    n = 5
    base_y, step = 178, 26
    for i in range(n):
        op = _prop([(0, 0), (max(0, i * 12 - 4), 0), (i * 12, 100), (OP - 8, 100), (OP, 0)])
        col = RED if i < n - 1 else RED_DEEP
        layers.append(_text(10 + i, _doc("stack", 22, col, tracking=8),
                            (W / 2, base_y - i * step), opacity=op, name=f"s{i}"))
    of_op = _prop([(0, 0), (OP - 30, 0), (OP - 18, 100), (OP - 6, 0), (OP, 0)])
    of_pos = _prop([(0, [W / 2, 34, 0]), (OP - 10, [W / 2, 12, 0]), (OP, [W / 2, 34, 0])])
    layers.append(_text(40, _doc("overflow", 16, RED_DEEP, tracking=10), (W / 2, 34),
                        position=of_pos, opacity=of_op, name="overflow"))
    return _comp("stack-overflow", layers)


def async_drop():
    """'async' sits; a message token flies off-screen and is dropped."""
    layers = [_kicker(1, "undeliverable")]
    layers.append(_text(10, _doc("async", 50, RED, tracking=8), (W / 2, 124), name="async"))
    pos = _prop([(0, [120, 124, 0]), (8, [120, 124, 0]), (40, [216, 104, 0]),
                 (48, [216, 104, 0]), (50, [120, 124, 0]), (OP, [120, 124, 0])])
    op = _prop([(0, 0), (8, 0), (16, 100), (38, 100), (46, 0), (OP, 0)])
    layers.append(_rect(20, (120, 124), [26, 16], RED_DEEP, position=pos, opacity=op,
                        roundness=3, name="msg"))
    drop_op = _prop([(0, 0), (40, 0), (46, 100), (60, 100), (70, 0), (OP, 0)])
    layers.append(_text(21, _doc("dropped", 14, RED_DEEP, tracking=12), (W / 2, 170),
                        opacity=drop_op, name="dropped"))
    return _comp("async-io-misuse", layers)


def pointer():
    """'*ptr ->' points at a block; the block frees, the pointer dangles."""
    layers = [_kicker(1, "use-after-free")]
    layers.append(_text(10, _doc("*ptr", 44, RED, tracking=6), (84, 132), name="ptr"))
    layers.append(_text(11, _doc("->", 38, RED_SOFT), (150, 130), name="arrow"))
    blk_op = _prop([(0, 100), (38, 100), (48, 0), (70, 0), (78, 100), (OP, 100)])
    layers.append(_rect(20, (194, 132), [30, 30], RED_DEEP, opacity=blk_op, roundness=5, name="block"))
    freed_op = _prop([(0, 0), (48, 0), (50, 100), (68, 100), (70, 0), (OP, 0)])
    layers.append(_text(21, _doc("freed", 13, RED_DEEP, tracking=8), (194, 172),
                        opacity=freed_op, name="freed"))
    return _comp("pointer-mismanagement", layers)


def invalidate():
    """CACHE; then 'IN' slides in to make INVALIDATION — the recall."""
    layers = [_kicker(1, "two hard problems")]
    layers.append(_text(10, _doc("CACHE", 40, LIGHT, tracking=10), (W / 2, 104), name="cache"))
    layers.append(_text(11, _doc("VALIDATION", 28, RED, justify=0, tracking=2), (84, 152), name="validation"))
    pos = _prop([(0, [44, 152, 0]), (6, [44, 152, 0]), (24, [84, 152, 0]),
                 (64, [84, 152, 0]), (80, [44, 152, 0]), (OP, [44, 152, 0])])
    op = _prop([(0, 0), (6, 0), (22, 100), (66, 100), (80, 0), (OP, 0)])
    layers.append(_text(12, _doc("IN", 28, RED_DEEP, justify=1), (84, 152),
                        position=pos, opacity=op, name="in"))
    return _comp("cache-invalidation", layers)


def mutex():
    """'mutex' guarding a value box whose state toggles LOCK / FREE."""
    layers = [_kicker(1, "critical section")]
    layers.append(_text(10, _doc("mutex", 46, RED, tracking=8), (W / 2, 116), name="mutex"))
    layers.append(_rect(20, (W / 2, 158), [72, 30], RED_DEEP, roundness=6, name="box"))
    st = _doc_seq([(0, "LOCK"), (OP // 2, "FREE")], 18, LIGHT, justify=2)
    layers.append(_text(21, st, (W / 2, 164), name="state"))
    return _comp("threading-mutexes", layers)


def network():
    """'network'; a packet hops between nodes, the last hop is dropped."""
    layers = [_kicker(1, "packets drop")]
    layers.append(_text(10, _doc("network", 40, RED, tracking=6), (W / 2, 108), name="network"))
    ny = 156
    for i, x in enumerate((50, 120, 190)):
        layers.append(_rect(20 + i, (x, ny), [16, 16], RED_DEEP, roundness=8, name=f"node{i}"))
    pos = _prop([(0, [50, ny, 0]), (20, [120, ny, 0]), (30, [120, ny, 0]),
                 (50, [170, ny, 0]), (56, [170, ny, 0]), (58, [50, ny, 0]), (OP, [50, ny, 0])])
    op = _prop([(0, 100), (48, 100), (52, 0), (57, 0), (58, 100), (OP, 100)])
    layers.append(_rect(30, (50, ny), [10, 10], RED_SOFT, position=pos, opacity=op,
                        roundness=5, name="packet"))
    drop_op = _prop([(0, 0), (50, 0), (52, 100), (70, 100), (74, 0), (OP, 0)])
    layers.append(_text(31, _doc("dropped", 12, RED_DEEP, tracking=8), (190, 184),
                        opacity=drop_op, name="dropped"))
    return _comp("file-network-access", layers)


def leak():
    """Open handles pile up with no close() — the count climbs until starvation."""
    layers = [_kicker(1, "open()")]
    # a stack of handles, one acquired every few frames, none ever released
    n = 5
    base, step = 196, 17
    for i in range(n):
        t_in = round((i + 1) * OP / (n + 2))
        op = _prop([(0, 0), (max(0, t_in - 3), 0), (t_in, 100), (OP, 100)], hold=False)
        y = base - i * step
        col = RED if i == n - 1 else RED_DEEP
        layers.append(_rect(20 + i, (W / 2, y), [104, 11], col, opacity=op,
                            roundness=2, name="fd%d" % i))
    # running count of leaked handles, never resets
    counts = [(0, "0")] + [(round((i + 1) * OP / (n + 2)), str(i + 1)) for i in range(n)]
    layers.append(_text(10, _doc_seq(counts, 34, RED_SOFT, justify=2),
                        (W / 2, 92), name="count"))
    layers.append(_text(11, _doc("never closed", 15, RED_SOFT, tracking=6),
                        (W / 2, 220), name="lbl"))
    return _comp("resource-leaks", layers)


def shared_family_glyph(lang):
    """shared-state family glyph: italic characters fading in and out in a
    traveling wave (red). Non-Latin scripts drop the italic face (fade only)."""
    latin = lang in LATIN_LANGS
    word = i18n.t(lang, "anim_shared")
    size = 46
    layout = _char_layout(word, size)
    n = max(1, len(layout))
    y = 130
    font = FONT_MONO_ITAL if latin else FONT_MONO
    layers, ind = [], 10
    for i, (ch, x) in enumerate(layout):
        if ch == " ":
            continue
        t0 = (i / n) * OP            # staggered dip -> wave across the word
        lo, hi = t0 - 11, t0 + 11
        op = _prop([(0, 100), (lo, 100), (t0, 15), (hi, 100), (OP, 100)])
        col = RED_SOFT if i % 2 == 0 else RED
        layers.append(_text(ind, _doc(ch, size, col, font=font), (x, y),
                            opacity=op, name="s%d" % i))
        ind += 1
    return _comp("shared-state", layers)


# ── unstructured-state treatments (blue, one per cause) ─────────────────────
def divide_by_zero():
    """'V / 0' with the result flickering between ∞ and NaN — an unguarded denominator."""
    layers = [_kicker(1, "I = V / R")]
    layers.append(_text(10, _doc("V /", 56, LIGHT, justify=0), (40, 126), name="vr"))
    pulse = _prop([(0, 100), (18, 100), (28, 35), (44, 100), (62, 35), (78, 100), (OP, 100)])
    layers.append(_text(11, _doc("0", 56, WARN, justify=0), (140, 126), opacity=pulse, name="zero"))
    res = _doc_seq([(0, "= ∞"), (OP // 2, "= NaN")], 38, BLUE_SOFT, justify=2)
    layers.append(_text(12, res, (W / 2, 184), name="res"))
    return _comp("divide-by-zero", layers)


def type_errors():
    """'"1" + 1' resolving to the surprising "11" then the intended 2 — coercion."""
    layers = [_kicker(1, "js · py · rb · sh")]
    layers.append(_text(10, _doc('"1" + 1', 50, LIGHT), (W / 2, 122), name="expr"))
    res = _doc_seq([(0, '= "11"'), (OP // 2, "= 2")], 40, WARN, justify=2)
    layers.append(_text(11, res, (W / 2, 182), name="res"))
    return _comp("type-errors", layers)


def index_oob():
    """An index ticks 0→3 past a 3-cell array; the read falls off into 'undefined'."""
    layers = [_kicker(1, "arr[i]")]
    layers.append(_text(10, _doc("[ a, b, c ]", 32, LIGHT), (W / 2, 100), name="arr"))
    idx = _doc_seq([(0, "i = 0"), (OP // 4, "i = 1"), (OP // 2, "i = 2"), (3 * OP // 4, "i = 3")],
                   26, BLUE_SOFT, justify=2)
    layers.append(_text(11, idx, (W / 2, 142), name="i"))
    val = _doc_seq([(0, "→ a"), (OP // 4, "→ b"), (OP // 2, "→ c"), (3 * OP // 4, "→ undefined")],
                   26, LIGHT, justify=2)
    layers.append(_text(12, val, (W / 2, 178), name="val"))
    oob = _prop([(0, 0), (3 * OP // 4 - 2, 0), (3 * OP // 4, 100), (OP - 4, 100), (OP, 0)])
    layers.append(_text(13, _doc("out of bounds", 15, WARN, tracking=10), (W / 2, 208),
                        opacity=oob, name="oob"))
    return _comp("index-out-of-bounds", layers)


def validate_rules():
    """'Reading → Writing → Executing' glow in sequence — the validation rules of thumb."""
    rows = ["Reading", "Writing", "Executing"]
    layers = [_kicker(1, "VALIDATE  INPUT")]
    span = OP // 3
    ys = [104, 140, 176]
    for i, r in enumerate(rows):
        a = i * span
        glow = _prop([(0, 35), (max(0, a - 4), 35), (a, 100),
                      (a + span - 6, 100), (a + span, 35), (OP, 35 if i else 100)])
        layers.append(_text(10 + i, _doc(r, 30, BLUE_SOFT, justify=0), (40, ys[i]),
                            opacity=glow, name=r))
    return _comp("lack-of-input-validation", layers)


def unconstrained():
    """Text typed into a fixed field keeps growing until it overflows the bounds."""
    layers = [_kicker(1, "no max length")]
    layers.append(_rect(5, (W / 2, 128), [148, 44], BLUE_DEEP, roundness=8, name="field"))
    seq = [(0, "a"), (12, "aaaa"), (24, "aaaaaaaa"), (36, "aaaaaaaaaaaa"),
           (48, "aaaaaaaaaaaaaaaaaa")]
    layers.append(_text(6, _doc_seq(seq, 22, LIGHT, justify=0), (50, 135), name="typed"))
    of = _prop([(0, 0), (40, 0), (46, 100), (OP - 6, 100), (OP, 0)])
    layers.append(_text(7, _doc("→ overflow", 18, WARN, justify=0), (50, 182),
                        opacity=of, name="of"))
    return _comp("unconstrained-inputs", layers)


def duck():
    """'duck.quack()' succeeds, then 'duck.fly()' fails — assumed shape, no contract."""
    layers = [_kicker(1, "if it quacks...")]
    call = _doc_seq([(0, "duck.quack()"), (OP // 2, "duck.fly()")], 30, LIGHT, justify=2)
    layers.append(_text(10, call, (W / 2, 124), name="call"))
    ok = _prop([(0, 100), (OP // 2 - 1, 100), (OP // 2, 0), (OP, 0)], hold=True)
    err = _prop([(0, 0), (OP // 2 - 1, 0), (OP // 2, 100), (OP, 100)], hold=True)
    layers.append(_text(11, _doc("✓ ok", 28, BLUE_SOFT), (W / 2, 178), opacity=ok, name="ok"))
    layers.append(_text(12, _doc("✗ no method", 26, WARN), (W / 2, 178), opacity=err, name="err"))
    return _comp("duck-typing-without-contracts", layers)


def missing_return():
    """'return' with the value 'undefined' dropping out the bottom and fading."""
    layers = [_kicker(1, "return ?")]
    layers.append(_text(10, _doc("return", 42, LIGHT), (W / 2, 110), name="ret"))
    pos = _prop([(0, [W / 2, 146, 0]), (38, [W / 2, 198, 0]), (46, [W / 2, 198, 0]),
                 (47, [W / 2, 146, 0]), (OP, [W / 2, 146, 0])], spatial=True)
    op = _prop([(0, 0), (8, 100), (34, 100), (44, 0), (47, 0), (OP, 0)])
    layers.append(_text(11, _doc("undefined", 30, WARN), (W / 2, 146),
                        position=pos, opacity=op, name="undef"))
    return _comp("missing-undefined-returns", layers)


def version_drift():
    """A version drifts 1.2.0 → 1.3.0 → 1.4.7 under a caret range; 'untested' fires."""
    layers = [_kicker(1, "^1.2.0")]
    seq = [(0, "v1.2.0"), (OP // 3, "v1.3.0"), (2 * OP // 3, "v1.4.7")]
    layers.append(_text(10, _doc_seq(seq, 46, LIGHT), (W / 2, 124), name="ver"))
    op = _prop([(0, 0), (2 * OP // 3 - 2, 0), (2 * OP // 3, 100), (OP - 6, 100), (OP, 0)])
    layers.append(_text(11, _doc("✗ untested", 24, WARN), (W / 2, 178), opacity=op, name="drift"))
    return _comp("version-library-mismanagement", layers)


def hydrate():
    """Untrusted bytes '{...}' hydrate into a live object that sprouts a shell."""
    layers = [_kicker(1, "loads()")]
    # the incoming bytes morph: raw -> live object -> executed payload
    morph = [
        (0, "{...}", FONT_MONO, BLUE_SOFT),
        (OP // 3, "obj", FONT_MONO, BLUE),
        (2 * OP // 3, "$ sh", FONT_MONO, WARN),
        (OP - 8, "$ sh", FONT_MONO, WARN),
    ]
    layers.append(_text(10, _doc_styled(morph, 46, justify=2), (W / 2, 132), name="bytes"))
    # caption flips from 'untrusted bytes' to 'RCE'
    cap = _doc_styled([
        (0, "untrusted bytes", FONT_MONO, BLUE_SOFT),
        (2 * OP // 3, "→ RCE", FONT_MONO, WARN),
    ], 18, justify=2)
    layers.append(_text(11, cap, (W / 2, 184), name="cap"))
    return _comp("insecure-deserialization", layers)


def unstructured_family_glyph(lang):
    """unstructured-state family glyph: the translated word with a per-letter
    vertical wobble (blue). No kicker, no 'STATE' label."""
    word = i18n.t(lang, "anim_unstructured")
    n_all = len(word)
    size = 44 if n_all <= 7 else 34 if n_all <= 10 else 26 if n_all <= 13 else 20
    layout = _char_layout(word, size)
    y = 124
    jit = [8, -7, 6, -9, 7, -5, 9, -6, 8, -7, 6, -8]
    layers, ind = [], 10
    for i, (ch, x) in enumerate(layout):
        if ch == " ":
            continue
        off = jit[i % len(jit)]
        pos = _prop([(0, [x, y + off, 0]), (30, [x, y - off, 0]),
                     (60, [x, y + off, 0]), (OP, [x, y + off, 0])], spatial=True)
        col = BLUE_SOFT if i % 2 == 0 else BLUE
        layers.append(_text(ind, _doc(ch, size, col), (x, y),
                            position=pos, name="u%d" % i))
        ind += 1
    return _comp("unstructured-state", layers)


def common_family_glyph(lang):
    """common-defects family glyph: the words 'common defects' where characters
    animate out into numbers tinted in the red/green/blue family shades — tying
    the common family to its three stateful siblings. Works in every script."""
    word = i18n.t(lang, "anim_common")
    parts = word.split()
    if len(parts) >= 2:
        lines = [parts[0], " ".join(parts[1:])]
    else:
        lines = [word]
    rgb = [RED, GREEN, BLUE]
    if len(lines) == 2:
        size, ys = 30, [104, 150]
    else:
        size, ys = 40, [128]
    # one global counter across both lines drives stagger, digit, and tint
    total = max(1, sum(len(ln.replace(" ", "")) for ln in lines))
    layers, ind, gidx = [], 10, 0
    for li, line in enumerate(lines):
        yl = ys[li]
        for ch, x in _char_layout(line, size):
            if ch == " ":
                continue
            t0 = 6 + (gidx / total) * (OP - 26)
            digit = str((gidx * 7 + 3) % 10)
            col = rgb[gidx % 3]
            frames = [
                (0, ch, FONT_MONO, GRAY),
                (t0, ch, FONT_MONO, GRAY),
                (t0 + 2, digit, FONT_MONO, col),
                (t0 + 12, digit, FONT_MONO, col),
                (t0 + 14, ch, FONT_MONO, GRAY),
                (OP, ch, FONT_MONO, GRAY),
            ]
            op = _prop([(0, 100), (t0, 100), (t0 + 2, 100),
                        (t0 + 12, 10), (t0 + 14, 100), (OP, 100)])
            pos = _prop([(0, [x, yl, 0]), (t0, [x, yl, 0]),
                         (t0 + 12, [x, yl - 16, 0]),
                         (t0 + 14, [x, yl, 0]), (OP, [x, yl, 0])], spatial=True)
            layers.append(_text(ind, _doc_styled(frames, size), (x, yl),
                                opacity=op, position=pos, name="c%d" % gidx))
            ind += 1
            gidx += 1
    return _comp("common-defects", layers)


def compound_family_glyph(lang):
    """compound-defects family glyph: every other family's signature motion in
    one word. Characters are assigned, round-robin, one of four treatments so the
    glyph 'utilizes all aspects of the other animations':
      0  mutated      -- green case-flip (lower->UPPER) + scale pop (entropy that
                         keeps changing the letters)
      1  unstructured -- blue char with an up/down wiggle that damps and settles
      2  shared       -- red italic char in a fade wave (left as-is)
      3  compound     -- a character that strobes/phases through the three child
                         colors (nondeterminism yellow, data-corruption magenta,
                         runtime-errors cyan)
    Non-Latin scripts keep the glyph and drop case-flip/italic (color + motion
    still read)."""
    latin = lang in LATIN_LANGS
    word = i18n.t(lang, "anim_compound")
    parts = word.split()
    lines = [parts[0], " ".join(parts[1:])] if len(parts) >= 2 else [word]
    if len(lines) == 2:
        size, ys = 30, [104, 150]
    else:
        size, ys = 40, [128]
    total = max(1, sum(len(ln.replace(" ", "")) for ln in lines))
    layers, ind, gidx = [], 10, 0

    # strobe timeline shared by treatment 3: phase through the three child colors,
    # flashing opacity at each swap. Independent of the per-char stagger.
    s = [OP // 6, OP // 3, OP // 2, 2 * OP // 3, 5 * OP // 6]
    strobe_cols = [ND_YEL, DC_MAG, RT_CYN, ND_YEL, DC_MAG, RT_CYN]

    for li, line in enumerate(lines):
        yl = ys[li]
        for ch, x in _char_layout(line, size):
            if ch == " ":
                continue
            t0 = 6 + (gidx / total) * (OP - 34)   # traveling-wave onset
            kind = gidx % 4
            alt = (gidx // 4) % 2

            if kind == 0:  # mutated: case-flip + scale pop, green
                up = ch.upper() if latin else ch
                hold = 6
                frames = [
                    (0, ch, FONT_MONO_REG, GREEN),
                    (t0, ch, FONT_MONO_REG, GREEN),
                    (t0 + 4, up, FONT_MONO, BRIGHT),
                    (t0 + 4 + hold, up, FONT_MONO, BRIGHT),
                    (t0 + 4 + hold + 5, ch, FONT_MONO_REG, GREEN),
                    (OP, ch, FONT_MONO_REG, GREEN),
                ]
                sc = _prop([(0, [100, 100, 100]), (t0, [100, 100, 100]),
                            (t0 + 4, [120, 120, 100]),
                            (t0 + 4 + hold, [120, 120, 100]),
                            (t0 + 4 + hold + 5, [100, 100, 100]),
                            (OP, [100, 100, 100])])
                layers.append(_text(ind, _doc_styled(frames, size), (x, yl),
                                    scale=sc, name="c%d" % gidx))

            elif kind == 1:  # unstructured: damped up/down wiggle that settles, blue
                col = BLUE_SOFT if alt == 0 else BLUE
                pos = _prop([(0, [x, yl, 0]), (t0, [x, yl, 0]),
                             (t0 + 4, [x, yl - 11, 0]), (t0 + 8, [x, yl + 8, 0]),
                             (t0 + 12, [x, yl - 5, 0]), (t0 + 16, [x, yl + 3, 0]),
                             (t0 + 20, [x, yl, 0]), (OP, [x, yl, 0])], spatial=True)
                sc = _prop([(0, [100, 100, 100]), (t0, [100, 100, 100]),
                            (t0 + 4, [113, 113, 100]), (t0 + 20, [100, 100, 100]),
                            (OP, [100, 100, 100])])
                layers.append(_text(ind, _doc(ch, size, col), (x, yl),
                                    position=pos, scale=sc, name="c%d" % gidx))

            elif kind == 2:  # shared: italic fade wave, red (unchanged treatment)
                font = FONT_MONO_ITAL if latin else FONT_MONO
                col = RED_SOFT if alt == 0 else RED
                lo, hi = t0 - 11, t0 + 11
                op = _prop([(0, 100), (lo, 100), (t0, 15), (hi, 100), (OP, 100)])
                layers.append(_text(ind, _doc(ch, size, col, font=font), (x, yl),
                                    opacity=op, name="c%d" % gidx))

            else:  # compound: strobe/phase through the three child colors
                frames = [(0, ch, FONT_MONO, strobe_cols[0])]
                for j, t in enumerate(s):
                    frames.append((t, ch, FONT_MONO, strobe_cols[j]))
                    frames.append((t + 1, ch, FONT_MONO, strobe_cols[j + 1]))
                frames.append((OP, ch, FONT_MONO, strobe_cols[-1]))
                okeys = [(0, 100)]
                for t in s:
                    okeys += [(t, 100), (t + 1, 45), (t + 4, 100)]
                okeys.append((OP, 100))
                layers.append(_text(ind, _doc_styled(frames, size), (x, yl),
                                    opacity=_prop(okeys), name="c%d" % gidx))

            ind += 1
            gidx += 1
    return _comp("compound-defects", layers)


# ── mutated-state replacement treatment (green) ─────────────────────────────
def aliasing_defaults():
    """`a` and `b` both point at one box; a write through b shows up in a."""
    layers = [_kicker(1, "a  =  b", color=DIM)]
    layers.append(_rect(5, (152, 140), [72, 50], DEEP, opacity=_static(100),
                        roundness=8, name="box"))
    vals = _doc_seq([(0, "[ ]"), (46, "[4]")], 30, LIGHT, justify=2)
    layers.append(_text(6, vals, (152, 150), name="val"))
    glow = _prop([(0, 60), (44, 60), (50, 100), (64, 100), (72, 60), (OP, 60)])
    layers.append(_text(10, _doc("a", 34, BRIGHT, justify=0), (40, 122),
                        opacity=glow, name="a"))
    layers.append(_text(11, _doc("b", 34, GREEN, justify=0), (40, 170),
                        opacity=glow, name="b"))
    layers.append(_text(12, _doc("→", 26, DIM, justify=0), (72, 122), name="ar_a"))
    layers.append(_text(13, _doc("→", 26, DIM, justify=0), (72, 170), name="ar_b"))
    return _comp("aliasing-mutable-defaults", layers)


# ── common-defects treatments (gray, one per child) ─────────────────────────
def copy_pasta():
    """'paste' stacks three times; the third copy drifts + discolors — drift."""
    layers = [_kicker(1, "COPY  ·  PASTE", color=GRAY_DEEP)]
    layers.append(_text(5, _doc("copy", 28, GRAY_SOFT, tracking=12), (W / 2, 80),
                        name="copy"))
    ys = [120, 150, 180]
    for i in range(3):
        a = 12 + i * 16
        op = _prop([(0, 0), (max(0, a - 6), 0), (a, 100), (OP - 8, 100), (OP, 0)])
        if i == 2:
            pos = _prop([(0, [W / 2, ys[i], 0]), (a, [W / 2, ys[i], 0]),
                         (OP - 8, [W / 2 + 18, ys[i], 0]), (OP, [W / 2 + 18, ys[i], 0])],
                        spatial=True)
            layers.append(_text(10 + i, _doc("paste", 28, WARN, tracking=12),
                                (W / 2, ys[i]), opacity=op, position=pos, name=f"p{i}"))
        else:
            layers.append(_text(10 + i, _doc("paste", 28, GRAY, tracking=12),
                                (W / 2, ys[i]), opacity=op, name=f"p{i}"))
    return _comp("copy-pasta", layers)


def dry_three():
    """Three identical rows collapse into one extracted call — rule of three."""
    layers = [_kicker(1, "RULE  OF  THREE", color=GRAY_DEEP)]
    ys = [96, 128, 160]
    for i in range(3):
        a = i * 12
        op = _prop([(0, 0), (max(0, a - 4), 0), (a, 100), (44, 100), (54, 0), (OP, 0)])
        layers.append(_text(10 + i, _doc("price = qty * unit", 17, GRAY_SOFT, tracking=4),
                            (W / 2, ys[i]), opacity=op, name=f"row{i}"))
    one_op = _prop([(0, 0), (54, 0), (62, 100), (OP - 6, 100), (OP, 0)])
    layers.append(_text(20, _doc("line_total()", 28, LIGHT, tracking=6), (W / 2, 132),
                        opacity=one_op, name="one"))
    tag_op = _prop([(0, 0), (60, 0), (66, 100), (OP, 100)])
    layers.append(_text(21, _doc("DRY", 16, WARN, tracking=120), (W / 2, 168),
                        opacity=tag_op, name="dry"))
    return _comp("dry-rule-of-three", layers)


def magic_numbers():
    """A bare 86400 with a blinking ? resolves into a named constant."""
    layers = [_kicker(1, "WHAT  IS...", color=GRAY_DEEP)]
    num_op = _prop([(0, 100), (44, 100), (52, 0), (OP, 0)])
    layers.append(_text(10, _doc("86400", 50, LIGHT, tracking=6), (W / 2, 140),
                        opacity=num_op, name="num"))
    q_op = _prop([(0, 0), (8, 0), (12, 100), (24, 100), (28, 0), (40, 0),
                  (44, 100), (50, 0), (OP, 0)], hold=True)
    layers.append(_text(11, _doc("?", 50, WARN, justify=0), (W / 2 + 70, 140),
                        opacity=q_op, name="q"))
    name_op = _prop([(0, 0), (52, 0), (60, 100), (OP - 6, 100), (OP, 0)])
    layers.append(_text(12, _doc("SECONDS_PER_DAY", 22, GRAY_SOFT, tracking=2),
                        (W / 2, 140), opacity=name_op, name="named"))
    return _comp("magic-numbers", layers)


def off_by_one_gray(word="error"):
    """'erro' + a trailing 'r' that slides one step off and fades — off by one."""
    stem, tail = word[:-1], word[-1]
    layers = [_kicker(1, "OFF  BY  ONE", color=GRAY_DEEP)]
    layers.append(_text(10, _doc(stem, 60, LIGHT, justify=0), (52, 138), name="stem"))
    pos = _prop([(0, [170, 138, 0]), (40, [210, 120, 0]), (44, [210, 120, 0]),
                 (45, [170, 138, 0]), (OP, [170, 138, 0])], spatial=True)
    op = _prop([(0, 100), (34, 100), (44, 0), (45, 0), (52, 100), (OP, 100)])
    layers.append(_text(11, _doc(tail, 60, WARN, justify=0), (170, 138),
                        position=pos, opacity=op, name="tail"))
    return _comp("off-by-one-errors", layers)


def naming_problems():
    """Vague names cycle and resolve into one good, intentional name."""
    layers = [_kicker(1, "NAME  THINGS", color=GRAY_DEEP)]
    seq = _doc_seq([(0, "x"), (20, "tmp"), (40, "data2"), (60, "userCount")],
                   40, GRAY_SOFT, tracking=2)
    layers.append(_text(10, seq, (W / 2, 140), name="name"))
    return _comp("naming-problems", layers)


def error_handling():
    """`catch (e) {}` — the caught error `e` shrinks away, swallowed."""
    layers = [_kicker(1, "SWALLOWED", color=GRAY_DEEP)]
    layers.append(_text(10, _doc("catch (", 32, GRAY_SOFT, justify=0), (24, 140),
                        name="lhs"))
    layers.append(_text(13, _doc(") {}", 32, GRAY_SOFT, justify=0), (176, 140),
                        name="rhs"))
    e_op = _prop([(0, 100), (30, 100), (48, 0), (64, 0), (72, 100), (OP, 100)])
    e_sc = _prop([(0, [100, 100, 100]), (48, [12, 12, 100]), (64, [12, 12, 100]),
                  (72, [100, 100, 100]), (OP, [100, 100, 100])])
    layers.append(_text(12, _doc("e", 32, WARN, justify=0), (156, 140),
                        opacity=e_op, scale=e_sc, name="e"))
    return _comp("error-handling", layers)


def dependency_issues():
    """An unpinned version drifts upward through releases until it breaks."""
    layers = [_kicker(1, "UNPINNED", color=GRAY_DEEP)]
    seq = _doc_seq([(0, "^1.2.3"), (20, "1.5.0"), (40, "1.9.3"), (62, "2.0.0")],
                   44, GRAY_SOFT, tracking=4)
    layers.append(_text(10, seq, (W / 2, 140), name="ver"))
    warn_op = _prop([(0, 0), (60, 0), (64, 100), (OP, 100)])
    layers.append(_text(11, _doc("!", 40, WARN, justify=0), (W / 2 + 70, 140),
                        opacity=warn_op, name="warn"))
    return _comp("dependency-issues", layers)


def injection_taint():
    """A tainted payload slides into a sink box that flashes on impact."""
    layers = [_kicker(1, "TAINT  →  SINK", color=GRAY_DEEP)]
    layers.append(_rect(5, (188, 140), [60, 50], GRAY_DEEP, opacity=_static(100),
                        roundness=8, name="sink"))
    layers.append(_text(6, _doc("DB", 22, GRAY_SOFT), (188, 148), name="db"))
    pos = _prop([(0, [36, 140, 0]), (40, [150, 140, 0]), (48, [150, 140, 0]),
                 (54, [36, 140, 0]), (OP, [36, 140, 0])], spatial=True)
    op = _prop([(0, 100), (40, 100), (46, 0), (52, 0), (58, 100), (OP, 100)])
    layers.append(_text(10, _doc("' OR 1=1", 22, WARN, justify=0), (36, 140),
                        position=pos, opacity=op, name="payload"))
    flash = _prop([(0, 0), (38, 0), (42, 80), (50, 0), (OP, 0)])
    layers.append(_rect(7, (188, 140), [60, 50], WARN, opacity=flash, roundness=8,
                        name="flash"))
    return _comp("injection-taint-flaws", layers)


def runtime_integration():
    """`GET /` resolves into a `500` — the failure a dynamic probe observes."""
    layers = [_kicker(1, "DAST  PROBE", color=GRAY_DEEP)]
    req_op = _prop([(0, 100), (36, 100), (44, 0), (OP, 0)])
    layers.append(_text(10, _doc("GET /", 44, GRAY_SOFT, tracking=4), (W / 2, 138),
                        opacity=req_op, name="req"))
    err_op = _prop([(0, 0), (44, 0), (52, 100), (OP - 6, 100), (OP, 0)])
    layers.append(_text(11, _doc("500", 56, WARN, tracking=8), (W / 2, 140),
                        opacity=err_op, name="err"))
    return _comp("runtime-integration-flaws", layers)


# slug → builder
TREATMENTS = {
    "var-let-mut": cycle,
    "aliasing-mutable-defaults": aliasing_defaults,
    "for-loop-control-flow": cascade,
    "this-mutation": slide,
    "time-money-entropy": ticker,
    "impure-functions": corrupt,
    "macro-mixin-misuse": expand,
    "increment-decrement-operators": counter,
    "improper-initialization": init_glitch,
    # common defects (gray)
    "copy-pasta": copy_pasta,
    "dry-rule-of-three": dry_three,
    "magic-numbers": magic_numbers,
    "off-by-one-errors": off_by_one_gray,
    "naming-problems": naming_problems,
    "error-handling": error_handling,
    "dependency-issues": dependency_issues,
    "injection-taint-flaws": injection_taint,
    "runtime-integration-flaws": runtime_integration,
    # shared-state causes
    "race-conditions": race,
    "resource-contention": contention,
    "stack-overflow": stack_of,
    "async-io-misuse": async_drop,
    "pointer-mismanagement": pointer,
    "cache-invalidation": invalidate,
    "threading-mutexes": mutex,
    "file-network-access": network,
    "resource-leaks": leak,
    # unstructured-state causes
    "divide-by-zero": divide_by_zero,
    "type-errors": type_errors,
    "index-out-of-bounds": index_oob,
    "lack-of-input-validation": validate_rules,
    "unconstrained-inputs": unconstrained,
    "duck-typing-without-contracts": duck,
    "missing-undefined-returns": missing_return,
    "version-library-mismanagement": version_drift,
    "insecure-deserialization": hydrate,
    # compound defects (gold) -- emergent from pairs of the families above;
    # they reuse on-brand treatments that echo each compound's smell.
    "nondeterminism": ticker,
    "runtime-errors": error_handling,
    "data-corruption": corrupt,
}


# family id -> language-parameterized builder (one Lottie file per language)
FAMILY_GLYPHS = {
    "mutated-state": mutated_family_glyph,
    "unstructured-state": unstructured_family_glyph,
    "shared-state": shared_family_glyph,
    "common-defects": common_family_glyph,
    "compound-defects": compound_family_glyph,
}


def build_all():
    """Language-agnostic sub-defect treatments (families are per-language)."""
    return {slug: fn() for slug, fn in TREATMENTS.items()}


def write_all(outdir):
    os.makedirs(outdir, exist_ok=True)
    written = []
    # sub-defect glyphs: code/word text stays English, written once
    for slug, data in build_all().items():
        with open(os.path.join(outdir, f"{slug}.json"), "w", encoding="utf-8") as f:
            json.dump(data, f, separators=(",", ":"))
        written.append(slug)
    # family glyphs: language-native words, one file per language under <lang>/
    for lang in i18n.LANGS:
        ldir = os.path.join(outdir, lang)
        os.makedirs(ldir, exist_ok=True)
        for fid, fn in FAMILY_GLYPHS.items():
            with open(os.path.join(ldir, f"{fid}.json"), "w", encoding="utf-8") as f:
                json.dump(fn(lang), f, separators=(",", ":"))
    # mark families generated so the build skips any legacy static copy
    written.extend(FAMILY_GLYPHS.keys())
    return written


if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    target = os.path.join(here, "_anim_preview")
    print("wrote:", ", ".join(write_all(target)))
    print("->", target)
