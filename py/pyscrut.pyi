from __future__ import annotations

from enum import Enum

class Output:
    stderr: bytes
    stdout: bytes
    exit_code: int | str

    def __init__(self, stdout: bytes, stderr: bytes, exit_code: int | str) -> None: ...

class OutputStreamControl(str, Enum):
    STDERR = "Stderr"
    STDOUT = "Stdout"
    COMBINED = "Combined"

class TestCaseWait:
    @property
    def timeout(self) -> int: ...
    @property
    def path(self) -> str | None: ...

class TestCaseConfig:
    @property
    def detached(self) -> bool | None: ...
    @property
    def environment(self) -> dict[str, str]: ...
    @property
    def keep_crlf(self) -> bool | None: ...
    @property
    def output_stream(self) -> OutputStreamControl | None: ...
    @property
    def skip_code(self) -> int | None: ...
    @property
    def timeout(self) -> int | None: ...
    @property
    def wait(self) -> TestCaseWait | None: ...

class DocumentConfig:
    @property
    def append(self) -> list[str] | None: ...
    @property
    def defaults(self) -> TestCaseConfig: ...
    @property
    def language_markers(self) -> list[str] | None: ...
    @property
    def prepend(self) -> list[str] | None: ...
    @property
    def shell(self) -> str | None: ...
    @property
    def total_timeout(self) -> int | None: ...

class TestCase:
    def __init__(
        self,
        title: str,
        shell_expression: str,
        exit_code: int,
        expectations: list[str],
        line_number: int | None,
        cram_compat: bool = False,
    ) -> None: ...
    @property
    def title(self) -> str: ...
    @property
    def shell_expression(self) -> str: ...
    @property
    def exit_code(self) -> int: ...
    @property
    def expectations(self) -> list[tuple[str, str, bool, bool]]: ...
    @property
    def config(self) -> TestCaseConfig: ...
    @property
    def line_number(self) -> int: ...
    def validate(
        self, output: Output, location: str | None = None
    ) -> tuple[bool, str]: ...

class CramParser:
    def __init__(self) -> None: ...
    def parse(self, text: str) -> tuple[DocumentConfig, list[TestCase]]: ...

class MarkdownParser:
    def __init__(self, languages: list[str]) -> None: ...
    @staticmethod
    def default() -> MarkdownParser: ...
    def parse(self, text: str) -> tuple[DocumentConfig, list[TestCase]]: ...
