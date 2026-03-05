import re
import shlex
from typing import Any

import pydantic
from pydantic import Field, model_validator

_VAR_NAME_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
_EXPR_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)(:\+|:-)(.*)$", re.DOTALL)


def _expand_template_arg(template: str, variables: dict[str, str]) -> str:
    """Expand constrained shell-like variable syntax in one argument template.

    Supported syntax:
    - $VAR
    - ${VAR}
    - ${VAR:+TEXT}
    - ${VAR:-DEFAULT}
    """

    def _expand(text: str) -> str:
        i = 0
        out: list[str] = []
        n = len(text)

        while i < n:
            ch = text[i]
            if ch != "$":
                out.append(ch)
                i += 1
                continue

            if i + 1 >= n:
                out.append("$")
                i += 1
                continue

            nxt = text[i + 1]
            if nxt == "$":
                out.append("$")
                i += 2
                continue

            if nxt == "{":
                end = text.find("}", i + 2)
                if end == -1:
                    raise ValueError(f"Unclosed template expression in argument: {template}")

                expr = text[i + 2 : end]
                op_match = _EXPR_RE.match(expr)
                if op_match:
                    var_name, operator, value_text = op_match.groups()
                    var_value = variables.get(var_name, "")
                    is_set = var_value != ""

                    if operator == ":+":
                        out.append(_expand(value_text) if is_set else "")
                    else:  # ':-'
                        out.append(var_value if is_set else _expand(value_text))
                else:
                    if not _VAR_NAME_RE.match(expr):
                        raise ValueError(f"Unsupported template expression: ${{{expr}}}")
                    out.append(variables.get(expr, ""))

                i = end + 1
                continue

            j = i + 1
            while j < n and (text[j].isalnum() or text[j] == "_"):
                j += 1

            var_name = text[i + 1 : j]
            if not var_name:
                out.append("$")
                i += 1
                continue

            out.append(variables.get(var_name, ""))
            i = j

        return "".join(out)

    return _expand(template)


class SaranAction(pydantic.BaseModel):
    """One non-shell subprocess action, represented as executable + argv."""

    executable: str
    args: list[str] = Field(default_factory=list)

    @model_validator(mode="before")
    @classmethod
    def from_single_mapping(cls, value: Any):
        """Accept {'cmd': ['arg1', 'arg2']} and normalize to model fields."""
        if isinstance(value, dict):
            if len(value) != 1:
                raise ValueError("Each item in 'actions' must be a single-key mapping like {'gh': ['pr', 'view']}")

            executable, args = next(iter(value.items()))
            if not isinstance(executable, str):
                raise ValueError("Action executable must be a string")
            if not isinstance(args, list) or not all(isinstance(arg, str) for arg in args):
                raise ValueError("Action arguments must be a list of strings")

            return {"executable": executable, "args": args}

        return value

    def to_argv(self, variables: dict[str, str]) -> list[str]:
        """Render constrained template expressions and remove empty argv elements."""
        rendered_parts = [_expand_template_arg(arg, variables) for arg in self.args]

        argv_parts: list[str] = []
        for part in rendered_parts:
            if part == "":
                continue

            # Allow conditionals like '${JSON:+--json $JSON}' to emit multiple argv tokens.
            split_parts = shlex.split(part, posix=True)
            argv_parts.extend(split_parts)

        return [self.executable, *argv_parts]
