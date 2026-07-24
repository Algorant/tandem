import json
import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).parents[1]))
from release_checks import select_workflow_run


FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "workflow-runs.json"


class WorkflowRunSelectionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.runs = json.loads(FIXTURE.read_text(encoding="utf-8"))

    def test_release_selector_requires_tag_and_release_commit(self):
        selected = select_workflow_run(
            "release",
            self.runs["release"],
            "tandem-v0.6.5",
            "release-commit",
            "",
        )
        self.assertEqual(selected["databaseId"], 9001)

    def test_aur_selector_uses_workflow_run_commit_and_post_release_boundary(self):
        release_completed_at = self.runs["release"][0]["updatedAt"]
        selected = select_workflow_run(
            "aur",
            self.runs["aur"],
            "tandem-v0.6.5",
            "release-commit",
            release_completed_at,
        )
        self.assertEqual(selected["databaseId"], 9104)
        self.assertEqual(selected["headBranch"], "main")


if __name__ == "__main__":
    unittest.main()
