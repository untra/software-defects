import hashlib
import unittest
from repair_preview import apply_recipe


def digest(source):
    return hashlib.sha256(source).hexdigest()


class RepairTests(unittest.TestCase):
    def recipe(self):
        return {"kind": "denominator-guard", "source_hash": digest(b"a / b"),
                "result_hash": digest(b"b ? a / b : null"),
                "edits": [{"range": [0, 5], "expected": "a / b", "replacement": "b ? a / b : null"}]}

    def test_reviewed_result(self):
        self.assertEqual(apply_recipe(b"a / b", self.recipe(), digest), b"b ? a / b : null")

    def test_stale_input(self):
        with self.assertRaisesRegex(ValueError, "stale"):
            apply_recipe(b"a / c", self.recipe(), digest)

    def test_precondition(self):
        recipe = self.recipe()
        recipe["edits"][0]["expected"] = "wrong"
        with self.assertRaisesRegex(ValueError, "precondition"):
            apply_recipe(b"a / b", recipe, digest)

    def test_overlapping_edits(self):
        recipe = self.recipe()
        recipe["edits"].append(recipe["edits"][0])
        with self.assertRaisesRegex(ValueError, "overlapping"):
            apply_recipe(b"a / b", recipe, digest)

    def test_unreviewed_result(self):
        recipe = self.recipe()
        recipe["result_hash"] = digest(b"wrong")
        with self.assertRaisesRegex(ValueError, "reviewed outcome"):
            apply_recipe(b"a / b", recipe, digest)
