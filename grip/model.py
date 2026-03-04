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


def _convert_to_bash_value(value: Any) -> str:
    """Convert a Python value to a bash-compatible string."""
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


class GripOption(pydantic.BaseModel):
    name: str
    bind_to: Optional[str] = None
    description: Optional[str] = None
    required: Optional[bool] = None
    default: Optional[Any] = None
    type: Optional[str] = None
    is_flag: Optional[bool] = None

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


class GripArgument(pydantic.BaseModel):
    name: str
    description: Optional[str] = None
    bind_to: Optional[str] = None
    description: Optional[str] = None
    required: Optional[bool] = None
    default: Optional[Any] = None

    def to_argument(self) -> Argument:
        arg = argument(self.name, required=self.required, default=self.default)
        return arg


class GripActionResult(pydantic.BaseModel):
    """Result of executing a grip action."""
    stdout: str = ""
    stderr: str = ""
    exit_code: int = 0


class GripCommand(pydantic.BaseModel):
    name: str
    action: str
    description: Optional[str] = None
    docstring: Optional[str] = None
    arguments: Optional[list[GripArgument]] = None
    options: Optional[list[GripOption]] = None
    subcommands: Optional[list["GripCommand"]] = None

    def execute_action(self, kwargs: dict) -> GripActionResult:
        """Execute the bash action with exported variables, return result."""
        action = self.action
        for arg in self.arguments or []:
            if arg.bind_to and arg.name in kwargs:
                action = f"export {arg.bind_to}='{_convert_to_bash_value(kwargs[arg.name])}'\n" + action
        for opt in self.options or []:
            if opt.bind_to and opt.name.lstrip("-") in kwargs:
                action = f"export {opt.bind_to}='{_convert_to_bash_value(kwargs[opt.name.lstrip('-')])}'\n" + action
        
        # Prepare environment to force color output
        env = os.environ.copy()
        env['FORCE_COLOR'] = '1'
        env['CLICOLOR_FORCE'] = '1'
        env['PY_COLORS'] = '1'
        
        with subprocess.Popen(
            ["bash", "-c", action], stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env
        ) as proc:
            stdout, stderr = proc.communicate()
            return GripActionResult(
                stdout=stdout.decode(),
                stderr=stderr.decode(),
                exit_code=proc.returncode,
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


class GripCLI(pydantic.BaseModel):
    name: str
    description: Optional[str] = None
    version: Optional[str] = None
    commands: Optional[list["GripCommand"]] = None

    def to_group(self) -> Group:
        def _main():
            pass

        grp = group(name=self.name, help=self.description)(_main)
        grp = version_option(version=self.version)(grp)

        for cmd in self.commands or []:
            grp.add_command(cmd.to_command())

        return grp
