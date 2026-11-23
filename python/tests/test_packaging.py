"""Packaging and production-readiness validation for the Python SDK.

This module owns two things:

1. Strict validation of the packaging artifacts (``pyproject.toml``, the
   package README, the packaging guide) and the release contract they encode
   (metadata, extras, version consistency).
2. A reusable :func:`validate_report` helper plus contract tests for the
   SDK-code features (type hints, ``py.typed``, report validation helper,
   timeout/cancel). Those features are owned by a separate change to
   ``python/gamerecomp_sdk/__init__.py``; when they are not present yet, the
   corresponding check skips with an explicit reason so this suite is green
   both before and after that change lands.
"""
from __future__ import annotations

import inspect
import re
import sys
import unittest
from pathlib import Path

try:  # tomllib is stdlib on 3.11+; `test` extra pulls tomli on 3.10.
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - exercised only on Python 3.10
    import tomli as tomllib  # type: ignore[no-redef]

REPO_ROOT = Path(__file__).resolve().parents[2]
_PY_SRC = str(REPO_ROOT / 'python')
if _PY_SRC not in sys.path:
    sys.path.insert(0, _PY_SRC)

import gamerecomp_sdk  # noqa: E402  (path set up above)
from gamerecomp_sdk import RecompileError  # noqa: E402

PYPROJECT = REPO_ROOT / 'pyproject.toml'
CARGO_TOML = REPO_ROOT / 'rust' / 'Cargo.toml'
PKG_README = REPO_ROOT / 'python' / 'README.md'
PKG_DIR = REPO_ROOT / 'python' / 'gamerecomp_sdk'
PACKAGING_DOC = REPO_ROOT / 'docs' / 'PYTHON_PACKAGING.md'

SDK_ERRORS = (AssertionError, ValueError, TypeError, RecompileError)


def load_pyproject() -> dict:
    """Parse the root ``pyproject.toml`` into a dictionary."""
    return tomllib.loads(PYPROJECT.read_text(encoding='utf-8'))


def validate_report(report: object, *, require_stages: bool = False) -> dict:
    """Validate the shape of a ``reports/report.json`` dictionary.

    Strict on types, tolerant of extra/forward-compatible fields. Returns the
    report unchanged on success and raises ``AssertionError`` with a
    descriptive message on the first violation.

    Args:
        report: The decoded report object.
        require_stages: Also require a non-empty ``stages`` list.

    Returns:
        The same ``report`` object (for convenient chaining in callers).
    """
    assert isinstance(report, dict), 'report must be a JSON object (dict)'
    assert 'score' in report, "report is missing required field 'score'"
    score = report['score']
    assert isinstance(score, int) and not isinstance(score, bool), \
        "'score' must be an int (not bool)"
    assert 0 <= score <= 100, f"'score' out of range [0, 100]: {score}"
    stages = report.get('stages', [])
    assert isinstance(stages, list), "'stages' must be a list"
    for index, stage in enumerate(stages):
        assert isinstance(stage, dict), f'stages[{index}] must be an object'
    if 'outcome' in report:
        assert isinstance(report['outcome'], dict), "'outcome' must be an object"
    if require_stages:
        assert stages, 'report has no stages'
    return report


class PackagingMetadataTest(unittest.TestCase):
    """Validate the build metadata we own in ``pyproject.toml``."""

    def test_pyproject_exists_and_parses(self):
        data = load_pyproject()
        self.assertIn('build-system', data)
        self.assertIn('project', data)

    def test_required_project_fields(self):
        project = load_pyproject()['project']
        for field in ('name', 'version', 'description', 'readme',
                      'requires-python', 'license', 'classifiers',
                      'optional-dependencies'):
            with self.subTest(field=field):
                self.assertIn(field, project, f'missing [project].{field}')
        self.assertEqual(project['name'], 'gamerecomp-sdk')
        self.assertRegex(project['version'], r'^\d+\.\d+\.\d+$')
        self.assertEqual(project['readme'], 'python/README.md')
        self.assertEqual(project['license'], 'AGPL-3.0-only')

    def test_requires_python_floor_matches_source_syntax(self):
        # Source uses PEP 604 unions and built-in generics -> 3.10 minimum.
        self.assertIn('>=3.10', load_pyproject()['project']['requires-python'])

    def test_build_backend_is_setuptools(self):
        build_system = load_pyproject()['build-system']
        self.assertEqual(build_system['build-backend'], 'setuptools.build_meta')
        self.assertTrue(any('setuptools' in req
                            for req in build_system['requires']))

    def test_no_runtime_dependencies(self):
        self.assertEqual(load_pyproject()['project'].get('dependencies', []), [])

    def test_extras_are_declared(self):
        extras = load_pyproject()['project']['optional-dependencies']
        for name in ('test', 'build', 'dev'):
            with self.subTest(extra=name):
                self.assertIn(name, extras)
        self.assertTrue(any(dep.startswith('pytest')
                            for dep in extras['test']))
        self.assertTrue(any(dep.startswith('build')
                            for dep in extras['build']))
        self.assertTrue(any(dep.startswith('twine')
                            for dep in extras['dev']))

    def test_package_discovery_is_rooted_at_python(self):
        find = load_pyproject()['tool']['setuptools']['packages']['find']
        self.assertEqual(find['where'], ['python'])
        self.assertIn('gamerecomp_sdk*', find['include'])

    def test_readme_is_present_and_substantive(self):
        self.assertTrue(PKG_README.is_file(), 'python/README.md is missing')
        self.assertGreater(len(PKG_README.read_text(encoding='utf-8')), 200)

    def test_packaging_guide_documents_opt_in(self):
        self.assertTrue(PACKAGING_DOC.is_file(),
                        'docs/PYTHON_PACKAGING.md is missing')
        self.assertIn('GAMERECOMP_RUN_REAL_INTEGRATION',
                      PACKAGING_DOC.read_text(encoding='utf-8'))

    def test_version_matches_rust_binding(self):
        py_version = load_pyproject()['project']['version']
        cargo = tomllib.loads(CARGO_TOML.read_text(encoding='utf-8'))
        self.assertEqual(cargo['package']['version'], py_version)


class ReportValidatorTest(unittest.TestCase):
    """Exercise the reusable report validation helper itself."""

    def test_accepts_valid_report(self):
        report = {'score': 93, 'stages': [{'name': 'inspect'}],
                  'outcome': {'status': 'ok'}}
        self.assertIs(validate_report(report, require_stages=True), report)

    def test_rejects_malformed_reports(self):
        bad_reports = (
            [],                                  # not an object
            {},                                  # no score
            {'score': 'high'},                   # wrong type
            {'score': True},                     # bool is not an int here
            {'score': -1},                       # out of range
            {'score': 101},                      # out of range
            {'score': 50, 'stages': 'nope'},     # stages not a list
            {'score': 50, 'stages': ['nope']},   # stage not an object
            {'score': 50, 'outcome': 'bad'},     # outcome not an object
        )
        for report in bad_reports:
            with self.subTest(report=report):
                with self.assertRaises(AssertionError):
                    validate_report(report)

    def test_require_stages_enforces_non_empty_stages(self):
        with self.assertRaises(AssertionError):
            validate_report({'score': 50, 'stages': []}, require_stages=True)


class SdkContractTest(unittest.TestCase):
    """Assert the production features supplied by the SDK-code change.

    Each SDK-side check skips explicitly if the feature is not present yet,
    because ``python/gamerecomp_sdk/__init__.py`` is owned by another change.
    """

    def test_sdk_version_matches_package_version(self):
        if not hasattr(gamerecomp_sdk, '__version__'):
            self.skipTest('gamerecomp_sdk.__version__ is supplied by the '
                          'SDK-code change')
        self.assertEqual(gamerecomp_sdk.__version__,
                         load_pyproject()['project']['version'])

    def test_public_methods_are_annotated(self):
        for name in ('__init__', 'inspect', 'recompile', 'recompile_to_path'):
            method = getattr(gamerecomp_sdk.Client, name)
            with self.subTest(method=name):
                signature = inspect.signature(method)
                for param_name, param in signature.parameters.items():
                    if param_name == 'self':
                        continue
                    self.assertIsNot(
                        param.annotation, inspect.Signature.empty,
                        f'Client.{name} parameter {param_name!r} is unannotated')
                if name != '__init__':  # constructors commonly omit -> None
                    self.assertIsNot(
                        signature.return_annotation, inspect.Signature.empty,
                        f'Client.{name} has no return annotation')

    def test_pep561_marker_is_packaged(self):
        marker = PKG_DIR / 'py.typed'
        if not marker.is_file():
            self.skipTest('py.typed marker is supplied by the SDK-code change')
        self.assertTrue(marker.is_file())
        package_data = load_pyproject()['tool']['setuptools']['package-data']
        self.assertIn('py.typed', package_data['gamerecomp_sdk'])

    def test_report_validation_helper_is_available(self):
        helper = getattr(gamerecomp_sdk, 'validate_report', None)
        if helper is None:
            self.skipTest('SDK validate_report() is supplied by the SDK-code '
                          'change; the reference helper is validate_report() '
                          'in this module')
        good = {'score': 70, 'stages': [{'name': 'report'}],
                'outcome': {'status': 'ok'}}
        helper(good)  # must not raise
        for bad in ({}, {'score': 'bad'}, {'score': 101},
                    {'score': 70, 'stages': 'bad'}):
            with self.subTest(bad=bad):
                with self.assertRaises(SDK_ERRORS):
                    helper(bad)

    def test_client_accepts_timeout(self):
        params = inspect.signature(gamerecomp_sdk.Client.__init__).parameters
        if 'timeout' not in params:
            self.skipTest('Client(timeout=...) is supplied by the SDK-code '
                          'change')
        default = params['timeout'].default
        self.assertTrue(
            default is None or (
                isinstance(default, (int, float))
                and not isinstance(default, bool) and default > 0),
            'timeout default must be a positive number of seconds or None')

    def test_client_supports_cancellation(self):
        cancel = getattr(gamerecomp_sdk.Client, 'cancel', None)
        if cancel is None:
            self.skipTest('Client.cancel() is supplied by the SDK-code change')
        self.assertTrue(callable(cancel))


if __name__ == '__main__':
    unittest.main()
