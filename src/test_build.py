"""Smoke test: the site builds and every stateful defect page renders its corpus."""
import os
import re
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO

import build
import corpus
import i18n

SAMPLE = ("mutated-state", "time-money-entropy")
CORPUS_TAB = re.compile(r'<button class="lang-tab[^"]*" data-lang="([a-z]+)" data-target="corpus-')


class BuildTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory(prefix="site-build-test-")
        with redirect_stdout(StringIO()):
            build.build(out=cls.tmp.name)
        cls.corpus = corpus.load_corpus()

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def page(self, lang, family, slug):
        with open(os.path.join(self.tmp.name, lang, family, slug, "index.html"), encoding="utf-8") as f:
            return f.read()

    def test_sample_page_has_one_tab_per_corpus_language_with_every_example(self):
        html = self.page("en", *SAMPLE)
        entry = self.corpus[SAMPLE]
        self.assertEqual(CORPUS_TAB.findall(html), [lg for lg, _ in entry["languages"]])
        for lg, examples in entry["languages"]:
            panel = html.split(f'id="corpus-{SAMPLE[0]}-{SAMPLE[1]}-{lg}"', 1)[1].split('<div class="code-panel', 1)[0]
            self.assertEqual(panel.count('<details class="corpus-ex"'), len(examples), lg)
        self.assertIn('class="hll"', html)

    def test_every_stateful_defect_page_in_every_locale_renders_its_corpus(self):
        for family, slug in self.corpus:
            for lang in i18n.LANGS:
                self.assertIn("corpus-viewer", self.page(lang, family, slug), f"{lang}/{family}/{slug}")

    def test_defects_without_corpus_render_no_corpus_section(self):
        for d in build.DEFECTS:
            for child in d.get("defects", []):
                if (d["id"], child["slug"]) not in self.corpus:
                    self.assertNotIn("corpus-viewer", self.page("en", d["id"], child["slug"]))


if __name__ == "__main__":
    unittest.main()
