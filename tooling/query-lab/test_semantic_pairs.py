import unittest
from semantic_pairs import witness


class SemanticTests(unittest.TestCase):
    def test_missing_relation_is_ambiguous(self):
        result = witness({"unsupported": [], "events": []}, {"pattern": "shared-nested-object", "contract": {"write": "test.db.host", "read": "prod.db.host"}})
        self.assertEqual(result["status"], "ambiguous")

    def test_unknown_effect_prevents_validation(self):
        self.assertEqual(witness({"unsupported": ["unknown call"], "events": []}, {})["status"], "unsupported")

    def test_distinct_property_does_not_establish_failure(self):
        events = [{"kind":"write", "path":"test.db.host", "object":1, "property":"host", "range":{"start":10,"end":20}}, {"kind":"read", "path":"prod.db.pool", "object":1, "property":"pool", "range":{"start":30,"end":40}}]
        spec = {"pattern":"shared-nested-object", "contract":{"write":"test.db.host","read":"prod.db.pool"}}
        self.assertFalse(witness({"unsupported":[],"events":events},spec)["failure"])

    def test_read_before_write_is_not_affected(self):
        events = [{"kind":"write", "path":"test.db.host", "object":1, "property":"host", "range":{"start":30,"end":40}}, {"kind":"read", "path":"prod.db.host", "object":1, "property":"host", "range":{"start":10,"end":20}}]
        spec = {"pattern":"shared-nested-object", "contract":{"write":"test.db.host","read":"prod.db.host"}}
        self.assertFalse(witness({"unsupported":[],"events":events},spec)["failure"])

    def test_missing_intent_contract_does_not_validate_shadowing(self):
        from semantic_pairs import compare
        result = compare({"id":"legitimate-shadowing", "expectations":{}}, None, None)
        self.assertEqual(result["status"], "unsupported")

    def test_unreviewed_contract_cannot_enable_repairs(self):
        from semantic_pairs import compare
        case = {"id":"case", "expectations":{"semantic":{"schema_version":1,"review_status":"draft"}}}
        self.assertEqual(compare(case,None,None)["status"], "unsupported")
