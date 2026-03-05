import os
import subprocess
import sys
from typing import Any, Optional

import pydantic
from click import (
    Argument,
    Command,
    Context,
    Group,
    Option,
    argument,
    command,
    group,
    option,
    pass_context,
    version_option,
)

from saran.action import SaranAction


def _convert_to_bash_value(value: Any) -> str:
    """Convert a Python value to an environment variable string."""
    if value is None:
        return ""
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value)


def _convert_click_type(type_str: Optional[str]) -> Optional[type]:
    """Convert a string type name to a Click type."""
    if type_str is None:
        return None
    if type_str == "str":
        return str
    if type_str == "int":
        return int
    if type_str == "float":
        return float
    if type_str == "bool":
        return bool
    # Default to string if unknown
    return str


class SaranOption(pydantic.BaseModel):
    name: str
    bind_to: Optional[str] = None
    description: Optional[str] = None
    required: Optional[bool] = None
    default: Optional[Any] = None
    type: Optional[str] = None
    is_flag: Optional[bool] = None

    def param_name(self) -> str:
        """Return the Click kwargs key for this option name."""
        return self.name.lstrip("-").replace("-", "_")

    def to_option(self) -> Option:
        is_flag = self.is_flag or (self.type == "bool" or (self.type is None and self.default in ("true", "false")))
        default_val = self.default in ("true", True) if is_flag else self.default
        click_type = _convert_click_type(self.type) if not is_flag else None
        opt = option(
            self.name,
            help=self.description,
            required=self.required,
            default=default_val,
            type=click_type,
            is_flag=is_flag,
        )
        return opt


class SaranArgument(pydantic.BaseModel):
    name: str
    description: Optional[str] = None
    bind_to: Optional[str] = None
    description: Optional[str] = None
    required: Optional[bool] = None
    default: Optional[Any] = None

    def param_name(self) -> str:
        """Return the Click kwargs key for this argument name."""
        return self.name.strip("<>").replace("-", "_")

    def to_argument(self) -> Argument:
        arg = argument(self.name, required=self.required, default=self.default)
        return arg


class SaranActionResult(pydantic.BaseModel):
    """Result of executing a saran action."""

    stdout: str = ""
    stderr: str = ""
    exit_code: int = 0


class SaranCommand(pydantic.BaseModel):
    name: str
    actions: list[SaranAction]
    description: Optional[str] = None
    docstring: Optional[str] = None
    arguments: Optional[list[SaranArgument]] = None
    options: Optional[list[SaranOption]] = None
    subcommands: Optional[list["SaranCommand"]] = None

    def _build_bound_environment(self, kwargs: dict[str, Any]) -> dict[str, str]:
        """Build environment variables defined by argument/option bind_to settings."""
        bound_env: dict[str, str] = {}

        for arg in self.arguments or []:
            key = arg.param_name()
            if arg.bind_to and key in kwargs:
                bound_env[arg.bind_to] = _convert_to_bash_value(kwargs[key])

        for opt in self.options or []:
            key = opt.param_name()
            if not opt.bind_to or key not in kwargs:
                continue

            value = kwargs[key]
            if opt.is_flag:
                # Preserve the semantic value of flags by using the CLI flag token.
                bound_env[opt.bind_to] = opt.name if value else ""
            else:
                bound_env[opt.bind_to] = _convert_to_bash_value(value)

        return bound_env

    def execute_action(self, kwargs: dict) -> SaranActionResult:
        """Execute actions sequentially using subprocess without invoking a shell."""
        # Prepare environment and keep colorized output behavior.
        env = os.environ.copy()
        env.update(self._build_bound_environment(kwargs))
        env["FORCE_COLOR"] = "1"
        env["CLICOLOR_FORCE"] = "1"
        env["PY_COLORS"] = "1"

        stdout_chunks: list[str] = []
        stderr_chunks: list[str] = []
        exit_code = 0

        for action in self.actions:
            argv = action.to_argv(env)
            result = subprocess.run(
                argv,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
                text=True,
                shell=False,
            )

            stdout_chunks.append(result.stdout)
            stderr_chunks.append(result.stderr)
            exit_code = result.returncode

            if result.returncode != 0:
                break

        return SaranActionResult(
            stdout="".join(stdout_chunks),
            stderr="".join(stderr_chunks),
            exit_code=exit_code,
        )

    def to_command(self) -> Command | Group:
        # If this command has subcommands, create a Group; otherwise create a Command
        if self.subcommands:

            def _group_callback(ctx: Context, *args, **kwargs):
                # Only execute the group's action if no subcommand was invoked
                if ctx.invoked_subcommand is not None:
                    return

                # Execute group's action
                result = self.execute_action(kwargs)
                if result.exit_code != 0:
                    print(result.stderr, file=sys.stderr)
                    sys.exit(result.exit_code)
                else:
                    print(result.stdout, end="")

            grp = group(name=self.name, help=self.description, invoke_without_command=True)(
                pass_context(_group_callback)
            )
            grp.__doc__ = self.docstring

            for arg in self.arguments or []:
                grp = arg.to_argument()(grp)

            for opt in self.options or []:
                grp = opt.to_option()(grp)

            # Recursively add subcommands
            for subcmd in self.subcommands:
                grp.add_command(subcmd.to_command())

            return grp
        else:

            def _command(ctx: Context, *args, **kwargs):
                # Execute the action
                result = self.execute_action(kwargs)
                if result.exit_code != 0:
                    print(result.stderr, file=sys.stderr)
                    sys.exit(result.exit_code)
                else:
                    print(result.stdout, end="")

            cmd = command(name=self.name, help=self.description)(pass_context(_command))
            cmd.__doc__ = self.docstring

            for arg in self.arguments or []:
                cmd = arg.to_argument()(cmd)

            for opt in self.options or []:
                cmd = opt.to_option()(cmd)

            return cmd


class SaranCLI(pydantic.BaseModel):
    name: str
    description: Optional[str] = None
    version: Optional[str] = None
    commands: Optional[list["SaranCommand"]] = None

    def to_group(self) -> Group:
        def _main():
            pass

        grp = group(name=self.name, help=self.description)(_main)
        grp = version_option(version=self.version)(grp)

        for cmd in self.commands or []:
            grp.add_command(cmd.to_command())

        return grp
