#!/usr/bin/env python3
"""Static-site generator for the Software Defects field guide.

Reads content.py (defect catalog + narrative) and i18n.py (UI labels), emits a
fully static multilingual site with Pygments-highlighted code, Lottie glyphs,
and the diagnostic-console design system in assets/css/style.css.

Output: ./_site/  (per-language dirs + shared /assets). Root index redirects to /en/.
Also dumps content -> _data/defects.yml and labels -> _data/i18n.yml for
architecture fidelity (the documented Jekyll data model).
"""
import html
import os
import shutil

import content
import corpus
import i18n
import anim_gen
from highlight import highlight_code, THEME_CSS

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.abspath(os.path.join(HERE, "..", "site"))
ASSETS_SRC = os.path.join(HERE, "assets")

DEFECTS = content.DEFECTS
CORPUS = corpus.load_corpus()
REPO_URL = "https://github.com/untra/software-defects/blob/main"
BY_ID = {d["id"]: d for d in DEFECTS}
ORDER = [d["id"] for d in DEFECTS]
# slug -> (family_id, child dict); lets compound defects resolve their
# ingredient links back to the existing defect pages.
SLUG_INDEX = {ch["slug"]: (d["id"], ch)
              for d in DEFECTS for ch in d.get("defects", [])}

CAT_OF = {
    "mutated-state": "mutated",
    "unstructured-state": "unstructured",
    "shared-state": "shared",
    "common-defects": "common",
    "compound-defects": "compound",
}

# Reverse of the recipe: stateful-defect slug -> the worked compound examples
# that use it as an ingredient. Powers the "where this compounds" cards on each
# stateful defect page. (Built once; each family feeds exactly two smells.)
INGREDIENT_USAGE = {}
for _smell in BY_ID.get("compound-defects", {}).get("defects", []):
    for _ex in _smell.get("examples", []):
        if not _ex.get("slug"):
            continue
        _ings = _ex.get("ingredients", [])
        for _slug in _ings:
            _other = next((s for s in _ings if s != _slug), None)
            INGREDIENT_USAGE.setdefault(_slug, []).append({
                "smell_slug": _smell["slug"],
                "smell_name": _smell["name"],
                "smell_color": _smell["color_primary"],
                "smell_deep": _smell["color_deep"],
                "example_slug": _ex["slug"],
                "example_title": _ex["title"],
                "other": _other,
            })

# Which compound smells each stateful family can feed (fixed domain facts).
FAMILY_SMELLS = {
    "mutated-state": ["nondeterminism", "runtime-errors"],
    "shared-state": ["nondeterminism", "data-corruption"],
    "unstructured-state": ["runtime-errors", "data-corruption"],
}

# ---------------------------------------------------------------- helpers
def e(s):
    return html.escape(str(s), quote=True)


def accent_style(d):
    # color_* are stored as bare hex; CSS consumes these via var(--accent) as a
    # color value, which requires the leading '#'.
    return (f"--accent:#{d['color_primary']};"
            f"--accent-soft:#{d['color_secondary']};"
            f"--accent-deep:#{d['color_deep']};")


def lottie_cdn():
    return ('<script src="https://cdnjs.cloudflare.com/ajax/libs/'
            'bodymovin/5.12.2/lottie.min.js" defer></script>')


def fonts_head():
    return (
        '<link rel="preconnect" href="https://fonts.googleapis.com">'
        '<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>'
        '<link href="https://fonts.googleapis.com/css2?'
        'family=Inter:wght@400;500;600&'
        'family=Space+Grotesk:wght@400;500;600;700&'
        'family=Noto+Sans+Arabic:wght@400;500;600;700&'
        'family=JetBrains+Mono:ital,wght@0,400;0,700;1,400;1,700&display=swap" rel="stylesheet">'
    )


GLYPH_SVG = (
    '<svg class="glyph" viewBox="0 0 24 24" fill="none" aria-hidden="true">'
    '<path d="M12 2 3 7v10l9 5 9-5V7z" stroke="currentColor" stroke-width="1.5" '
    'stroke-linejoin="round"/><path d="M12 12 3 7M12 12l9-5M12 12v10" '
    'stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" opacity=".5"/>'
    '<circle cx="12" cy="12" r="2.4" fill="currentColor"/></svg>'
)
ARROW_SVG = ('<svg viewBox="0 0 24 24" fill="none" aria-hidden="true">'
             '<path d="M5 12h14M13 6l6 6-6 6" stroke="currentColor" '
             'stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>')


# ---------------------------------------------------------------- chrome
def header(lang, active):
    L = lambda k: e(i18n.t(lang, k))
    home = f"/{lang}/"
    links = []
    nav_map = [
        ("mutated-state", "nav_mutated", "mutated"),
        ("unstructured-state", "nav_unstructured", "unstructured"),
        ("shared-state", "nav_shared", "shared"),
        ("common-defects", "nav_common", "common"),
        ("compound-defects", "nav_compound", "compound"),
    ]
    for did, key, cat in nav_map:
        cur = ' aria-current="page"' if active == did else ""
        links.append(f'<a class="nav--{cat}" href="/{lang}/{did}/"{cur}>{L(key)}</a>')
    links_html = "\n        ".join(links)

    opts = []
    # language switcher: same page, swap lang prefix
    target_suffix = f"{active}/" if active and active != "home" else ""
    for lc in i18n.LANGS:
        sel = " selected" if lc == lang else ""
        opts.append(f'<option value="/{lc}/{target_suffix}"{sel}>{e(i18n.LANG_LABEL[lc])}</option>')
    opts_html = "".join(opts)

    return f"""<header class="site-header">
  <div class="wrap nav">
    <a class="brand" href="{home}">{GLYPH_SVG}<span>Software&nbsp;Defects<small>{L('brand_tagline')}</small></span></a>
    <button class="nav-toggle" aria-label="Menu" aria-expanded="false">
      <svg viewBox="0 0 24 24" fill="none"><path d="M4 7h16M4 12h16M4 17h16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>
    </button>
    <nav class="nav-links">
        {links_html}
        <select class="lang-select" aria-label="Language" onchange="location.href=this.value">{opts_html}</select>
    </nav>
  </div>
</header>"""


def footer(lang):
    L = lambda k: e(i18n.t(lang, k))
    links = " ".join(
        f'<a href="/{lang}/{did}/">{e(i18n.t(lang, "nav_" + CAT_OF[did]))}</a>'
        for did in ORDER
    )
    return f"""<footer class="site-footer">
  <div class="wrap site-footer__inner">
    <p>{L('footer_note')} <em>Samuel&nbsp;Volin</em>. Families of stateful defect, the smells that betray them, and the philosophies that defuse them.</p>
    <nav class="flinks">{links}</nav>
  </div>
</footer>"""


def page(lang, active, title, body, extra_head=""):
    return f"""<!doctype html>
<html lang="{lang}" dir="{i18n.dir_of(lang)}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{e(title)}</title>
<meta name="description" content="A field guide to the families of stateful software defect — mutated, unstructured, shared, common, and the compound defects that emerge between them.">
{fonts_head()}
<link rel="stylesheet" href="/assets/css/style.css">
<link rel="stylesheet" href="/assets/css/code-highlight.css">
{extra_head}
</head>
<body>
{header(lang, active)}
<main>
{body}
</main>
{footer(lang)}
<script src="/assets/js/app.js" defer></script>
</body>
</html>"""


# ---------------------------------------------------------------- code viewer
def code_viewer(example):
    langs = example["languages"]
    tabs, panels = [], []
    base = "cv" + str(abs(hash(example["title"])) % 10_000_000)
    for i, lg in enumerate(langs):
        act = " is-active" if i == 0 else ""
        slug = lg["slug"]
        pid = f"{base}-{slug}"
        tabs.append(
            f'<button class="lang-tab{act}" data-lang="{e(slug)}" '
            f'data-target="{pid}" role="tab" aria-selected="{"true" if i==0 else "false"}" '
            f'type="button">{e(lg["name"])}</button>'
        )
        hl = highlight_code(lg["code"], slug)
        fname = e(filename_for(slug))
        out = ""
        if lg.get("output"):
            out = (f'<div class="code-out"><span class="tag">{e("// observed")}</span>'
                   f'<pre>{e(lg["output"])}</pre></div>')
        explain = ""
        if lg.get("explanation"):
            explain = f'<div class="code-explain">{e(lg["explanation"])}</div>'
        panels.append(
            f'<div class="code-panel{act}" id="{pid}" data-lang="{e(slug)}">'
            f'<div class="code-head"><span class="fname">{fname}</span>'
            f'<button class="copy-btn" data-copied="copied" type="button">copy</button></div>'
            f'<pre><code class="hl">{hl}</code></pre>'
            f'{explain}{out}</div>'
        )
    return (
        f'<div class="code-viewer" data-cv="{base}">'
        f'<div class="code-window"><span class="dots"><i></i><i></i><i></i></span>'
        f'<div class="lang-tabs">{"".join(tabs)}</div></div>'
        f'{"".join(panels)}</div>'
    )


# ---------------------------------------------------------------- corpus viewer
def _corpus_code(lang, code, slug, fname, hl_lines=()):
    return (
        f'<div class="corpus-code"><div class="code-head"><span class="fname">{e(fname)}</span>'
        f'<button class="copy-btn" data-copied="copied" type="button">copy</button></div>'
        f'<pre><code class="hl">{highlight_code(code, slug, hl_lines)}</code></pre></div>'
    )


def _corpus_example(lang, slug, ex, is_open):
    L = lambda k: e(i18n.t(lang, k))
    fname = os.path.basename(ex["file"])
    badge = ("" if ex["deterministic"]
             else f'<span class="corpus-ex__badge">{L("nondeterministic")}</span>')
    fixed = ""
    if ex["fixed_source"]:
        why = (f'<div class="code-explain">{e(ex["fix_rationale"])}</div>'
               if ex["fix_rationale"] else "")
        fixed = (f'<details class="corpus-fix"><summary>{L("fixed_version")}</summary>'
                 f'{why}{_corpus_code(lang, ex["fixed_source"], slug, fname)}</details>')
    return (
        f'<details class="corpus-ex"{" open" if is_open else ""}>'
        f'<summary><span class="fname">{e(fname)}</span>{badge}'
        f'<span class="corpus-ex__summary">{e(ex["summary"])}</span></summary>'
        f'{_corpus_code(lang, ex["source"], slug, fname, ex["bug_lines"])}'
        f'<div class="code-explain">{e(ex["mechanism"])}</div>'
        f'<div class="code-out"><span class="tag">{L("observed")}</span><pre>{e(ex["observed_output"])}</pre>'
        f'<span class="tag">{L("expected")}</span><pre>{e(ex["expected_behavior"])}</pre></div>'
        f'{fixed}'
        f'<a class="corpus-ex__src" href="{REPO_URL}/{e(ex["path"])}">{L("view_source")}</a>'
        f'</details>'
    )


def corpus_viewer(lang, family_id, child_slug, entry):
    tabs, panels = [], []
    base = f"corpus-{family_id}-{child_slug}"
    for i, (slug, examples) in enumerate(entry["languages"]):
        act = " is-active" if i == 0 else ""
        pid = f"{base}-{slug}"
        tabs.append(
            f'<button class="lang-tab{act}" data-lang="{e(slug)}" data-target="{pid}" role="tab" '
            f'aria-selected="{"true" if i == 0 else "false"}" type="button">'
            f'{e(corpus.language_label(slug))} · {len(examples)}</button>'
        )
        items = "".join(_corpus_example(lang, slug, ex, j == 0) for j, ex in enumerate(examples))
        panels.append(f'<div class="code-panel corpus-panel{act}" id="{pid}" data-lang="{e(slug)}">{items}</div>')
    return (
        f'<div class="code-viewer corpus-viewer" data-cv="{base}">'
        f'<div class="code-window"><span class="dots"><i></i><i></i><i></i></span>'
        f'<div class="lang-tabs">{"".join(tabs)}</div></div>'
        f'{"".join(panels)}</div>'
    )


EXT = {
    "javascript": "js", "typescript": "ts", "python": "py", "ruby": "rb",
    "go": "go", "rust": "rs", "java": "java", "kotlin": "kt", "c": "c",
    "bash": "sh", "solidity": "sol", "hcl": "tf",
}


def filename_for(slug):
    return f"example.{EXT.get(slug, 'txt')}"


# ---------------------------------------------------------------- homepage
def cascade_strip():
    c = content.CASCADE
    cells = []
    stages = c["stages"]
    for i, st in enumerate(stages):
        cells.append(
            f'<div class="cascade__cell"><span class="cascade__num">0{i+1}</span>'
            f'<h4>{e(st["label"])}</h4><p>{e(st["note"])}</p></div>'
        )
        if i < len(stages) - 1:
            cells.append(f'<div class="cascade__arrow">{ARROW_SVG}</div>')
    # final "vulnerability becomes exploit" cell
    return f'<div class="cascade">{"".join(cells)}</div>'


def homepage(lang):
    L = lambda k: e(i18n.t(lang, k))
    c = content.CASCADE

    cards = []
    for d in DEFECTS:
        cat = CAT_OF[d["id"]]
        syms = "".join(f"<li>{e(s)}</li>" for s in d["symptoms"][:3])
        cards.append(f"""    <a class="card card--{cat}" href="/{lang}/{d['id']}/">
      <div class="card__glyph"><span class="lottie-glyph" data-src="/assets/animations/{lang}/{d['id']}.json"></span></div>
      <div class="card__body">
        <span class="tag card__kicker">{e(d['tagline'])}</span>
        <h3 class="card__title">{e(d['name'])}</h3>
        <p class="card__desc">{e(d['description'])}</p>
        <ul class="card__symptoms">{syms}</ul>
        <span class="card__link">{L('learn_more')} {ARROW_SVG}</span>
      </div>
    </a>""")
    cards_html = "\n".join(cards)

    exploits = "".join(
        f'<div class="context__cell"><span class="tag">cascade</span>'
        f'<p><strong>{e(x["defect"])}</strong> becomes {e(x["becomes"])}.</p></div>'
        for x in c["exploits"]
    )

    two = content.TWO_HARD_PROBLEMS
    two_items = "".join(f"<li>{e(x)}</li>" for x in two)

    hero = f"""<section class="console">
  <div class="wrap hero">
    <p class="tag hero__eyebrow">{L('hero_eyebrow')}</p>
    <h1 class="hero__title">{L('hero_title_a')} <span class="stink">{L('hero_title_b')}</span></h1>
    <p class="hero__sub">{L('hero_sub')}</p>
    {cascade_strip()}
  </div>
</section>"""

    catalog = f"""<section class="section">
  <div class="wrap">
    <div class="section__head"><span class="tag">{L('defects_tag')}</span><h2>{L('defects_title')}</h2></div>
    <div class="cards">
{cards_html}
    </div>
  </div>
</section>"""

    context = f"""<section class="section section--dark context">
  <div class="wrap">
    <div class="section__head"><span class="tag">the cascade</span><h2>This is why we can't have nice things</h2></div>
    <p>{e(c['intro'])} A defect that ships does not stay a defect. It becomes something an attacker can use.</p>
    <div class="context__grid">{exploits}</div>
  </div>
</section>"""

    two_hard = f"""<section class="two-hard">
  <div class="wrap two-hard__inner">
    <span class="two-hard__lead">// there are two hard problems in computer science:</span>
    <span class="two-hard__list"><ol>{two_items}</ol></span>
  </div>
</section>"""

    body = hero + catalog + context + two_hard
    title = "Software Defects — a field guide to stateful bugs"
    return page(lang, "home", title, body, extra_head=lottie_cdn())


# ---------------------------------------------------------------- defect page
def block(num, accent_label, heading, inner, intro=""):
    intro_html = f'<p class="block__intro">{intro}</p>' if intro else ""
    return f"""<section class="block">
  <div class="block__head"><span class="num">{e(num)}</span><span class="tag">{e(accent_label)}</span><h2>{e(heading)}</h2></div>
  {intro_html}
  {inner}
</section>"""


def cwes_html(cwes):
    """Linked list of mapped CWEs; reuses the .phil-list styling."""
    lis = "".join(
        f'<li><a href="{e(c["url"])}" target="_blank" rel="noopener">'
        f'<b>{e(c["id"])}</b> &mdash; {e(c["name"])}</a></li>'
        for c in cwes
    )
    return f'<ul class="phil-list">{lis}</ul>'


def cwe_chip(c, extra=""):
    """One CWE chip: id + optional shortname, full title as hover tooltip."""
    short = content.CWE_SHORT.get(c["id"], "")
    label = (f'{e(c["id"])} <span class="recipe__cwe-short">{e(short)}</span>'
             if short else e(c["id"]))
    return (f'<a class="recipe__cwe{extra}" href="{e(c["url"])}" target="_blank" '
            f'rel="noopener" title="{e(c["name"])}">{label}</a>')


def recipe_html(lang, ex):
    """For a compound defect example: the 'A x B' ingredient pair, each
    ingredient tinted by its source family and trailed by the base CWE(s) it
    contributes (so you can see which originating bug carried which weakness),
    then an arrow to the compound (chain/composite) CWE the pair forms."""
    cwes = ex.get("cwes", [])
    base = [c for c in cwes if c.get("role") != "compound"]
    comp = [c for c in cwes if c.get("role") == "compound"]

    # base CWEs grouped by the ingredient that contributed them
    base_by_from = {}
    for c in base:
        base_by_from.setdefault(c.get("from"), []).append(c)

    # one group per ingredient: the tinted link + the base chips it carries
    groups = []
    ings = ex.get("ingredients", [])
    for slug in ings:
        ref = SLUG_INDEX.get(slug)
        if not ref:
            continue
        fam_id, ch = ref
        cat = CAT_OF[fam_id]
        ing = (f'<a class="recipe__ing recipe__ing--{cat}" '
               f'href="/{lang}/{fam_id}/{slug}/">{e(ch["name"])}</a>')
        mine = base_by_from.get(slug, [])
        chips = (f'<span class="recipe__ing-cwes recipe__ing--{cat}">'
                 f'{"".join(cwe_chip(c) for c in mine)}</span>') if mine else ""
        groups.append(f'<span class="recipe__group">{ing}{chips}</span>')
    pair = '<span class="recipe__x" aria-hidden="true">&times;</span>'.join(groups)

    # any base CWE whose 'from' didn't match an ingredient — migration fallback
    orphan = [c for fr, items in base_by_from.items() if fr not in ings
              for c in items]
    orphan_html = (f'<span class="recipe__cwe-grp">'
                   f'<span class="recipe__cwe-lbl">{e(i18n.t(lang, "cwe_base"))}</span>'
                   f'{"".join(cwe_chip(c) for c in orphan)}</span>') if orphan else ""

    comp_html = (f'<span class="recipe__cwe-grp">'
                 f'<span class="recipe__cwe-lbl">{e(i18n.t(lang, "cwe_compound"))}</span>'
                 f'{"".join(cwe_chip(c, " recipe__cwe--compound") for c in comp)}</span>'
                 ) if comp else ""
    arrow = ('<span class="recipe__cwe-arrow" aria-hidden="true">&rarr;</span>'
             if comp_html else "")
    tail = (f'<span class="recipe__cwes">{orphan_html}{arrow}{comp_html}</span>'
            if (orphan_html or comp_html) else "")

    return (f'<div class="recipe"><span class="recipe__lbl">'
            f'{e(i18n.t(lang, "compound_of"))}</span>'
            f'<span class="recipe__pair">{pair}</span>{tail}</div>')


def pullquote_html(pq):
    cite = f"<cite>{e(pq['attribution'])}</cite>" if pq.get("attribution") else ""
    return f'<figure class="pullquote"><q>{e(pq["text"])}</q>{cite}</figure>'


def casefile_html(cs):
    pts = "".join(f"<li>{e(p)}</li>" for p in cs["points"])
    link = ""
    if cs.get("link"):
        link = f'<a class="casefile__link" href="{e(cs["link"])}" target="_blank" rel="noopener">{e("read the post-mortem ↗")}</a>'
    kicker = f'<p class="casefile__kicker">{e(cs["kicker"])}</p>' if cs.get("kicker") else ""
    return f"""<div class="casefile">
  <div class="casefile__head"><h3>{e(cs['title'])}</h3><span class="meta">{e(cs['meta'])}</span></div>
  <div class="casefile__body">
    <p>{e(cs['body'])}</p>
    <ul class="casefile__points">{pts}</ul>
    {kicker}{link}
  </div>
</div>"""


def rules_html(r):
    cards = "".join(
        f'<div class="rules__card"><h4>{e(g["verb"])}</h4><p>{e(g["expect"])}</p></div>'
        for g in r["groups"]
    )
    return f'<div class="rules">{cards}</div>'


def ctable_html(tbl, kind):
    rows = []
    if kind == "analogy":
        head = '<div class="ctable__row ctable__head"><div>Computer</div><div>Human</div><div>Why it\'s the same problem</div></div>'
        for r in tbl["rows"]:
            rows.append(f'<div class="ctable__row"><div class="ctable__a">{e(r["computer"])}</div>'
                        f'<div class="ctable__b">{e(r["human"])}</div><div class="ctable__n">{e(r["note"])}</div></div>')
    elif kind == "deadlock":
        head = '<div class="ctable__row ctable__head"><div>Failure</div><div>Like</div><div>What happens</div></div>'
        for r in tbl["rows"]:
            rows.append(f'<div class="ctable__row"><div class="ctable__a">{e(r["name"])}</div>'
                        f'<div class="ctable__b">{e(r["human"])}</div><div class="ctable__n">{e(r["desc"])}</div></div>')
    else:  # spectrum
        head = '<div class="ctable__row ctable__head"><div>Model</div><div>Languages</div><div>How failure behaves</div></div>'
        for r in tbl["rows"]:
            rows.append(f'<div class="ctable__row"><div class="ctable__a">{e(r["model"])}</div>'
                        f'<div class="ctable__b">{e(r["langs"])}</div><div class="ctable__n">{e(r["trait"])}</div></div>')
    cap = f'<div class="ctable__cap">{e(tbl["caption"])}</div>' if tbl.get("caption") else ""
    return f'<div class="ctable">{cap}{head}{"".join(rows)}</div>'


def qualities_html(nq):
    items = "".join(
        f'<div class="qualities__item"><b>{e(name)}</b><span>{e(desc)}</span></div>'
        for name, desc in nq["items"]
    )
    rule = f'<p class="qualities__rule">{e(nq["rule"])}</p>' if nq.get("rule") else ""
    return f'<div class="qualities">{items}</div>{rule}'


def subdefect_grid(lang, family):
    """Hub grid of child-defect cards, each with its own text Lottie glyph."""
    L = lambda k: e(i18n.t(lang, k))
    cards = []
    for c in family["defects"]:
        # a child may carry its own accent (compound defects do); fall back to family
        cstyle = f' style="{accent_style(c)}"' if c.get("color_primary") else ""
        cards.append(
            f'<a class="subcard" href="/{lang}/{family["id"]}/{c["slug"]}/"{cstyle}>'
            f'<span class="subcard__glyph lottie-glyph" '
            f'data-src="/assets/animations/{c["slug"]}.json"></span>'
            f'<span class="subcard__body">'
            f'<span class="tag subcard__kicker">{e(c["tagline"])}</span>'
            f'<b class="subcard__title">{e(c["name"])}</b>'
            f'<span class="subcard__essence">{e(c["essence"])}</span>'
            f'<span class="card__link">{L("read_defect")} {ARROW_SVG}</span>'
            f'</span></a>'
        )
    return f'<div class="subgrid">{"".join(cards)}</div>'


def compounds_into_html(lang, c):
    """For a stateful child defect: one card per compound smell it feeds, each
    listing the worked compound examples that use it as an ingredient (so the
    reader can walk up from a base bug to the compound defects it composes)."""
    uses = INGREDIENT_USAGE.get(c["slug"], [])
    if not uses:
        return ""
    groups = {}
    for u in uses:
        g = groups.setdefault(u["smell_slug"], {"name": u["smell_name"],
                              "color": u["smell_color"], "deep": u["smell_deep"],
                              "items": []})
        g["items"].append(u)
    cards = []
    for ss, g in groups.items():
        lis = []
        for u in g["items"]:
            other = ""
            if u["other"] and u["other"] in SLUG_INDEX:
                other = (f' <span class="compounds-card__x">&times; '
                         f'{e(SLUG_INDEX[u["other"]][1]["name"])}</span>')
            lis.append(f'<li><a href="/{lang}/compound-defects/{ss}/{u["example_slug"]}/">'
                       f'{e(u["example_title"])}</a>{other}</li>')
        cards.append(
            f'<div class="compounds-card" style="--c:#{e(g["color"])};--cd:#{e(g["deep"])}">'
            f'<a class="compounds-card__head" href="/{lang}/compound-defects/{ss}/">'
            f'{e(g["name"])}</a>'
            f'<ul class="compounds-card__list">{"".join(lis)}</ul></div>')
    return f'<div class="compounds-grid">{"".join(cards)}</div>'


def subdefect_page(lang, family, c):
    """Detail page for one child defect: hero + examples + sibling/back nav."""
    L = lambda k: e(i18n.t(lang, k))
    cat = CAT_OF[family["id"]]
    # a child may carry its own accent (compound defects do); fall back to family
    style = accent_style(c if c.get("color_primary") else family)

    crumb = (f'<a href="/{lang}/{family["id"]}/">{e(family["name"])}</a>'
             f'<span aria-hidden="true">/</span><span>{e(c["name"])}</span>')

    hero = f"""<section class="console dhero" style="{style}">
  <div class="wrap dhero__grid">
    <div>
      <nav class="dcrumb tag">{crumb}</nav>
      <p class="tag dhero__kicker">{e(c['tagline'])}</p>
      <h1 class="dhero__title">{e(c['name'])}</h1>
      <p class="dhero__desc">{e(c['description'])}</p>
      <p class="dhero__essence">{e(c['essence'])}</p>
    </div>
    <div class="dhero__art"><span class="lottie-glyph" data-src="/assets/animations/{c['slug']}.json"></span></div>
  </div>
</section>"""

    # compound smells link out to a page per worked example; stateful defects
    # render their examples inline as before.
    if cat == "compound" and any(ex.get("slug") for ex in c["examples"]):
        def _excard(ex):
            recipe = recipe_html(lang, ex) if ex.get("ingredients") else ""
            href = f'/{lang}/{family["id"]}/{c["slug"]}/{ex["slug"]}/'
            return (
                f'<div class="excard">'
                f'<a class="excard__title" href="{href}">{e(ex["title"])}</a>'
                f'<p class="excard__intro">{e(ex["intent"])}</p>{recipe}'
                f'<a class="card__link" href="{href}">{L("read_defect")} {ARROW_SVG}</a>'
                f'</div>'
            )
        # compound smells group their worked examples under named failure-vector
        # categories (a category tier between smell and example); fall back to a
        # flat grid when a smell carries no `categories`.
        by_slug = {ex["slug"]: ex for ex in c["examples"] if ex.get("slug")}
        cats = c.get("categories")
        if cats:
            seen, groups = set(), []
            for cg in cats:
                members = [by_slug[s] for s in cg.get("members", []) if s in by_slug]
                seen.update(ex["slug"] for ex in members)
                cards = "".join(_excard(ex) for ex in members)
                groups.append(
                    f'<div class="excat">'
                    f'<h3 class="excat__title">{e(cg["name"])}</h3>'
                    f'<p class="excat__core">{e(cg["core"])}</p>'
                    f'<div class="exgrid">{cards}</div></div>'
                )
            # three category columns, each stacking its worked-example cards
            examples_inner = f'<div class="excat-cols">{"".join(groups)}</div>'
            leftover = [ex for ex in c["examples"]
                        if ex.get("slug") and ex["slug"] not in seen]
            if leftover:  # any uncategorized example renders full-width below the columns
                cards = "".join(_excard(ex) for ex in leftover)
                examples_inner += f'<div class="excat"><div class="exgrid">{cards}</div></div>'
        else:
            examples_inner = f'<div class="exgrid">{"".join(_excard(ex) for ex in c["examples"])}</div>'
    else:
        ex_html = []
        for ex in c["examples"]:
            recipe = recipe_html(lang, ex) if ex.get("ingredients") else ""
            ex_html.append(
                f'<div class="example"><h3 class="example__title">{e(ex["title"])}</h3>'
                f'<p class="example__intro">{e(ex["intent"])}</p>{recipe}{code_viewer(ex)}</div>'
            )
        examples_inner = "\n".join(ex_html)
    examples = block("01", "in the wild", i18n.t(lang, "examples"), examples_inner)

    numbers = (f"{n:02d}" for n in range(2, 10))

    # runnable corpus programs for this defect, one tab per programming language
    corpus_block = ""
    entry = CORPUS.get((family["id"], c["slug"]))
    if entry:
        corpus_block = block(next(numbers), "runnable corpus", i18n.t(lang, "corpus_examples"),
                             corpus_viewer(lang, family["id"], c["slug"], entry),
                             intro=i18n.t(lang, "corpus_intro"))

    # where this stateful defect compounds — links up to the compound smells it
    # feeds (stateful pages only; compound smells don't compound further here)
    compounds_block = ""
    if cat != "compound":
        ci = compounds_into_html(lang, c)
        if ci:
            compounds_block = block(next(numbers), "cross-pollination",
                                    i18n.t(lang, "compounds_into"), ci)

    # mapped CWEs — this defect on its own, as a catalogued weakness
    cwe_block = ""
    if c.get("cwes"):
        cwe_block = block(next(numbers), "weakness catalog",
                          i18n.t(lang, "cwes"),
                          cwes_html(c["cwes"]),
                          intro=i18n.t(lang, "cwes_intro"))

    # sibling prev/next within the family + back to the family hub
    kids = family["defects"]
    idx = next(i for i, k in enumerate(kids) if k["slug"] == c["slug"])
    prv = kids[(idx - 1) % len(kids)]
    nxt = kids[(idx + 1) % len(kids)]
    xcards = [
        f'<a class="xnav--{cat}" href="/{lang}/{family["id"]}/{prv["slug"]}/">'
        f'<span class="tag">{L("prev")}</span><b>{e(prv["name"])}</b></a>',
        f'<a class="xnav--{cat}" href="/{lang}/{family["id"]}/">'
        f'<span class="tag">{L("back")}</span><b>{e(family["name"])}</b></a>',
        f'<a class="xnav--{cat}" href="/{lang}/{family["id"]}/{nxt["slug"]}/">'
        f'<span class="tag">{L("next")}</span><b>{e(nxt["name"])}</b></a>',
    ]
    xnav = f'<div class="wrap"><nav class="xnav">{"".join(xcards)}</nav></div>'

    body = (hero
            + f'<div class="section defect-body" style="{style}"><div class="wrap">'
            + examples + corpus_block + compounds_block + cwe_block + "</div></div>"
            + f'<div class="section" style="{style};padding-top:0">{xnav}</div>')

    title = f"{c['name']} — {family['name']}"
    return page(lang, family["id"], title, body, extra_head=lottie_cdn())


def compound_example_page(lang, family, smell, ex):
    """Detail page for one worked compound-defect example: smell-accented hero +
    the recipe (ingredients → compound CWE) + code + per-example CWE catalog +
    prev/next within the smell."""
    L = lambda k: e(i18n.t(lang, k))
    style = accent_style(smell)

    crumb = (f'<a href="/{lang}/{family["id"]}/">{e(family["name"])}</a>'
             f'<span aria-hidden="true">/</span>'
             f'<a href="/{lang}/{family["id"]}/{smell["slug"]}/">{e(smell["name"])}</a>'
             f'<span aria-hidden="true">/</span><span>{e(ex["title"])}</span>')

    hero = f"""<section class="console dhero" style="{style}">
  <div class="wrap dhero__grid">
    <div>
      <nav class="dcrumb tag">{crumb}</nav>
      <p class="tag dhero__kicker">{e(smell['tagline'])}</p>
      <h1 class="dhero__title">{e(ex['title'])}</h1>
      <p class="dhero__desc">{e(ex['intent'])}</p>
    </div>
    <div class="dhero__art"><span class="lottie-glyph" data-src="/assets/animations/{smell['slug']}.json"></span></div>
  </div>
</section>"""

    recipe = recipe_html(lang, ex) if ex.get("ingredients") else ""
    example_block = block("01", "the recipe", i18n.t(lang, "examples"),
                          f'<div class="example">{recipe}{code_viewer(ex)}</div>')

    cwe_block = ""
    if ex.get("cwes"):
        cwe_block = block("02", "weakness catalog", i18n.t(lang, "cwes"),
                          cwes_html(ex["cwes"]),
                          intro=i18n.t(lang, "cwes_intro"))

    # prev/next across this smell's worked examples + back to the smell hub
    exs = [x for x in smell["examples"] if x.get("slug")]
    idx = next(i for i, x in enumerate(exs) if x["slug"] == ex["slug"])
    prv = exs[(idx - 1) % len(exs)]
    nxt = exs[(idx + 1) % len(exs)]
    base = f'/{lang}/{family["id"]}/{smell["slug"]}/'
    xcards = [
        f'<a class="xnav--compound" href="{base}{prv["slug"]}/">'
        f'<span class="tag">{L("prev")}</span><b>{e(prv["title"])}</b></a>',
        f'<a class="xnav--compound" href="{base}">'
        f'<span class="tag">{L("back")}</span><b>{e(smell["name"])}</b></a>',
        f'<a class="xnav--compound" href="{base}{nxt["slug"]}/">'
        f'<span class="tag">{L("next")}</span><b>{e(nxt["title"])}</b></a>',
    ]
    xnav = f'<div class="wrap"><nav class="xnav">{"".join(xcards)}</nav></div>'

    body = (hero
            + f'<div class="section defect-body" style="{style}"><div class="wrap">'
            + example_block + cwe_block + "</div></div>"
            + f'<div class="section" style="{style};padding-top:0">{xnav}</div>')

    title = f"{ex['title']} — {smell['name']}"
    return page(lang, family["id"], title, body, extra_head=lottie_cdn())


def defect_page(lang, d):
    L = lambda k: e(i18n.t(lang, k))
    cat = CAT_OF[d["id"]]
    style = accent_style(d)

    # hero
    hero = f"""<section class="console dhero" style="{style}">
  <div class="wrap dhero__grid">
    <div>
      <p class="tag dhero__kicker">{e(d['tagline'])}</p>
      <h1 class="dhero__title">{e(d['name'])}</h1>
      <p class="dhero__desc">{e(d['description'])}</p>
      <p class="dhero__essence">{e(d['essence'])}</p>
    </div>
    <div class="dhero__art"><span class="lottie-glyph" data-src="/assets/animations/{lang}/{d['id']}.json"></span></div>
  </div>
</section>"""

    blocks = []
    n = 1

    # 01 — the defects grid first: the animated signature, in a consistent spot
    # right under the hero (or family-level examples for a leaf family). No intro
    # paragraph here — the old "this family breaks down into…" copy is dropped.
    if d.get("defects"):
        blocks.append(block(f"{n:02d}", "the defects", i18n.t(lang, "subdefects"),
                            subdefect_grid(lang, d))); n += 1
    else:
        ex_html = []
        for ex in d["examples"]:
            ex_html.append(
                f'<div class="example"><h3 class="example__title">{e(ex["title"])}</h3>'
                f'<p class="example__intro">{e(ex["intent"])}</p>{code_viewer(ex)}</div>'
            )
        blocks.append(block(f"{n:02d}", "in the wild", i18n.t(lang, "examples"),
                            "\n".join(ex_html))); n += 1

    # 02 — in practice: one theme paragraph (the tl;dr), then how it shows up
    # (symptoms) and the tools that make it easy. "What causes it" is dropped:
    # the defects above ARE the causes, and the paragraph says so.
    syms = "".join(f"<li>{e(s)}</li>" for s in d["symptoms"])
    tools = "".join(f"<span>{e(s)}</span>" for s in d["tools_hurt"])
    sublabel = lambda label, mt: (f'<p class="tag" style="margin:{mt} 0 .7rem;'
                                  f'color:var(--accent-deep)">{e(label)}</p>')
    practice_inner = (sublabel(i18n.t(lang, "symptoms"), ".2rem")
                      + f'<ul class="symptoms">{syms}</ul>'
                      + sublabel(i18n.t(lang, "tools_hurt"), "1.7rem")
                      + f'<div class="chips">{tools}</div>')
    blocks.append(block(f"{n:02d}", "the smell", i18n.t(lang, "practice"),
                        practice_inner, intro=e(d.get("practice", "")))); n += 1

    # special narrative blocks per category (after the core backbone)
    if d.get("pullquote"):
        pq = d["pullquote"]
        blocks.append(block(f"{n:02d}", pq.get("tag", "the dream"),
                            pq.get("heading", "The pure-function fantasy"),
                            pullquote_html(pq))); n += 1
    if d.get("analogy_table"):
        blocks.append(block(f"{n:02d}", "computer / human", "Every shared-state bug has a human name",
                            ctable_html(d["analogy_table"], "analogy"))); n += 1
    if d.get("name_qualities"):
        blocks.append(block(f"{n:02d}", "naming things", d["name_qualities"]["title"],
                            qualities_html(d["name_qualities"]))); n += 1

    # post-example special blocks
    if d.get("case_study"):
        blocks.append(block(f"{n:02d}", "case file", "When it ships anyway",
                            casefile_html(d["case_study"]))); n += 1
    if d.get("rules"):
        blocks.append(block(f"{n:02d}", "rules of thumb", d["rules"]["title"],
                            rules_html(d["rules"]))); n += 1
    if d.get("deadlock_table"):
        blocks.append(block(f"{n:02d}", "stalls", "Two ways concurrency achieves nothing",
                            ctable_html(d["deadlock_table"], "deadlock"))); n += 1
    if d.get("spectrum"):
        blocks.append(block(f"{n:02d}", "execution model", "How far you are from the metal",
                            ctable_html(d["spectrum"], "spectrum"))); n += 1

    # meta: philosophies
    phils = "".join(
        f'<li><a href="{e(p["url"])}" target="_blank" rel="noopener">{e(p["name"])}</a></li>'
        for p in d["philosophies"]
    )
    pillar_html = ""
    if d.get("cwe_pillar"):
        p = d["cwe_pillar"]
        pillar_html = (
            f'<p class="block__intro" style="margin-top:1.4rem">'
            f'{e(i18n.t(lang, "cwe_pillar"))} '
            f'<a href="{e(p["url"])}" target="_blank" rel="noopener">'
            f'<b>{e(p["id"])}</b> &mdash; {e(p["name"])}</a></p>'
        )
    meta = block(f"{n:02d}", "antidotes", i18n.t(lang, "philosophies"),
                 f'<ul class="phil-list">{phils}</ul>{pillar_html}'); n += 1

    # cross-nav
    idx = ORDER.index(d["id"])
    nxt = BY_ID[ORDER[(idx + 1) % len(ORDER)]]
    prv = BY_ID[ORDER[(idx - 1) % len(ORDER)]]
    xcards = []
    for role, dd in (("prev", prv), ("next", nxt)):
        c2 = CAT_OF[dd["id"]]
        xcards.append(f'<a class="xnav--{c2}" href="/{lang}/{dd["id"]}/">'
                      f'<span class="tag">{L(role)}</span><b>{e(dd["name"])}</b></a>')
    all_card = (f'<a class="xnav--{cat}" href="/{lang}/">'
                f'<span class="tag">{L("back")}</span><b>{L("view_all")}</b></a>')
    xnav = f'<div class="wrap"><nav class="xnav">{xcards[0]}{all_card}{xcards[1]}</nav></div>'

    body_inner = "\n".join(blocks) + "\n" + meta
    body = (hero
            + f'<div class="section defect-body" style="{style}"><div class="wrap">'
            + body_inner + "</div></div>"
            + f'<div class="section" style="{style};padding-top:0">{xnav}</div>')

    title = f"{d['name']} — Software Defects"
    return page(lang, d["id"], title, body, extra_head=lottie_cdn())


# ---------------------------------------------------------------- data dumps
def dump_data():
    import json
    ddir = os.path.join(OUT, "_data")
    os.makedirs(ddir, exist_ok=True)
    # defects.yml — emit as readable YAML via a tiny serializer to preserve architecture
    try:
        import yaml
        with open(os.path.join(ddir, "defects.yml"), "w") as f:
            yaml.safe_dump({"defects": DEFECTS, "cascade": content.CASCADE,
                            "two_hard_problems": content.TWO_HARD_PROBLEMS},
                           f, allow_unicode=True, sort_keys=False, width=100)
        with open(os.path.join(ddir, "i18n.yml"), "w") as f:
            yaml.safe_dump(i18n.I18N, f, allow_unicode=True, sort_keys=False, width=100)
    except Exception as ex:
        with open(os.path.join(ddir, "defects.json"), "w") as f:
            json.dump({"defects": DEFECTS, "cascade": content.CASCADE}, f, indent=2, ensure_ascii=False)
        with open(os.path.join(ddir, "i18n.json"), "w") as f:
            json.dump(i18n.I18N, f, indent=2, ensure_ascii=False)


# ---------------------------------------------------------------- build
def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def build(out=None):
    global OUT
    OUT = os.path.abspath(out) if out else OUT
    if os.path.exists(OUT):
        shutil.rmtree(OUT)
    os.makedirs(OUT)

    # assets
    shutil.copytree(ASSETS_SRC, os.path.join(OUT, "assets"))
    # code highlight theme
    write(os.path.join(OUT, "assets", "css", "code-highlight.css"), THEME_CSS)
    # animations — generated programmatically (text-based Lottie glyphs).
    # Sub-defects are language-agnostic; the family glyphs are emitted once
    # per language under assets/animations/<lang>/ using language-native words.
    anim_out = os.path.join(OUT, "assets", "animations")
    anim_gen.write_all(anim_out)

    # pages
    for lang in i18n.LANGS:
        write(os.path.join(OUT, lang, "index.html"), homepage(lang))
        for d in DEFECTS:
            write(os.path.join(OUT, lang, d["id"], "index.html"), defect_page(lang, d))
            for child in d.get("defects", []):
                write(os.path.join(OUT, lang, d["id"], child["slug"], "index.html"),
                      subdefect_page(lang, d, child))
                # compound smells: one page per worked example, nested a level deeper
                if d["id"] == "compound-defects":
                    for ex in child.get("examples", []):
                        if not ex.get("slug"):
                            continue
                        write(os.path.join(OUT, lang, d["id"], child["slug"],
                                           ex["slug"], "index.html"),
                              compound_example_page(lang, d, child, ex))

    # GitHub Pages: serve every file verbatim (no Jekyll processing)
    write(os.path.join(OUT, ".nojekyll"), "")
    # GitHub Pages: custom domain (served from the domain root)
    write(os.path.join(OUT, "CNAME"), "badsoftware.untra.io\n")

    # root redirect
    write(os.path.join(OUT, "index.html"),
          '<!doctype html><meta charset="utf-8">'
          '<meta http-equiv="refresh" content="0; url=/en/">'
          '<link rel="canonical" href="/en/"><title>Software Defects</title>'
          '<a href="/en/">Software Defects →</a>')

    dump_data()

    # report
    pages = sum(len(files) for _, _, files in os.walk(OUT))
    print(f"built -> {OUT}")
    print(f"languages: {', '.join(i18n.LANGS)}")
    print(f"defects:   {', '.join(ORDER)}")
    print(f"files:     {pages}")
    coverage_report()


def coverage_report():
    """Compound-defect variety check: per stateful family, which child defects
    are not yet used as an ingredient in each of the two smells they can feed.
    Authors mine the gaps to broaden coverage. Report-only — emits nothing."""
    print("compound coverage (unused ingredients per smell):")
    for fam_id, smells in FAMILY_SMELLS.items():
        kids = [ch["slug"] for ch in BY_ID[fam_id]["defects"]]
        for ss in smells:
            used = {slug for slug, uses in INGREDIENT_USAGE.items()
                    if any(u["smell_slug"] == ss for u in uses) and slug in kids}
            missing = [k for k in kids if k not in used]
            status = ", ".join(missing) if missing else "(full coverage)"
            print(f"  {fam_id:>18} -> {ss:<15} {len(used)}/{len(kids)} used"
                  f"{'' if not missing else '  unused: ' + status}")


if __name__ == "__main__":
    build()
