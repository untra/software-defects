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
import i18n
import anim_gen
from highlight import highlight_code, THEME_CSS

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.abspath(os.path.join(HERE, "..", "site"))
ASSETS_SRC = os.path.join(HERE, "assets")

DEFECTS = content.DEFECTS
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


def recipe_html(lang, ex):
    """For a compound defect example: the 'A x B' ingredient link pair (each
    tinted by its source family) plus the chained/composed CWE chips."""
    ings = []
    for slug in ex.get("ingredients", []):
        ref = SLUG_INDEX.get(slug)
        if not ref:
            continue
        fam_id, ch = ref
        cat = CAT_OF[fam_id]
        ings.append(
            f'<a class="recipe__ing recipe__ing--{cat}" '
            f'href="/{lang}/{fam_id}/{slug}/">{e(ch["name"])}</a>'
        )
    pair = '<span class="recipe__x" aria-hidden="true">&times;</span>'.join(ings)

    # CWEs split into the base weaknesses each ingredient carries and the
    # compound (chain/composite) weakness their combination forms.
    cwes = ex.get("cwes", [])
    base = [c for c in cwes if c.get("role") != "compound"]
    comp = [c for c in cwes if c.get("role") == "compound"]

    def cwe_grp(items, key, extra=""):
        if not items:
            return ""
        chips = "".join(
            f'<a class="recipe__cwe{extra}" href="{e(c["url"])}" target="_blank" '
            f'rel="noopener" title="{e(c["name"])}">{e(c["id"])}</a>'
            for c in items
        )
        lbl = f'<span class="recipe__cwe-lbl">{e(i18n.t(lang, key))}</span>'
        return f'<span class="recipe__cwe-grp">{lbl}{chips}</span>'

    base_html = cwe_grp(base, "cwe_base")
    comp_html = cwe_grp(comp, "cwe_compound", extra=" recipe__cwe--compound")
    arrow = ('<span class="recipe__cwe-arrow" aria-hidden="true">&rarr;</span>'
             if base_html and comp_html else "")
    chips_html = (f'<span class="recipe__cwes">{base_html}{arrow}{comp_html}</span>'
                  if (base_html or comp_html) else "")
    return (f'<div class="recipe"><span class="recipe__lbl">'
            f'{e(i18n.t(lang, "compound_of"))}</span>'
            f'<span class="recipe__pair">{pair}</span>{chips_html}</div>')


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

    ex_html = []
    for ex in c["examples"]:
        recipe = recipe_html(lang, ex) if ex.get("ingredients") else ""
        ex_html.append(
            f'<div class="example"><h3 class="example__title">{e(ex["title"])}</h3>'
            f'<p class="example__intro">{e(ex["intent"])}</p>{recipe}{code_viewer(ex)}</div>'
        )
    examples = block("01", "in the wild", i18n.t(lang, "examples"), "\n".join(ex_html))

    # mapped CWEs — this defect on its own, as a catalogued weakness
    cwe_block = ""
    if c.get("cwes"):
        cwe_block = block("02", "weakness catalog", i18n.t(lang, "cwes"),
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
            + examples + cwe_block + "</div></div>"
            + f'<div class="section" style="{style};padding-top:0">{xnav}</div>')

    title = f"{c['name']} — {family['name']}"
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


def build():
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

    # GitHub Pages: serve every file verbatim (no Jekyll processing)
    write(os.path.join(OUT, ".nojekyll"), "")
    # GitHub Pages: custom domain (served from the domain root)
    write(os.path.join(OUT, "CNAME"), "softwaredefects.untra.io\n")

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


if __name__ == "__main__":
    build()
