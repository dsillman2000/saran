import sys
from pathlib import Path

import yaml_reference
from pydantic import BaseModel, ValidationError

from saran.model import SaranCLI


def main():
    """Read a YAML file and parse it into SaranCLI model."""
    try:
        input_file = sys.argv[1]
        file_path = Path(input_file)
        
        # Set $SARAN to the absolute path of the CLI script
        import os
        os.environ['SARAN'] = str(file_path.resolve())
        
        data = yaml_reference.load_yaml_with_references(file_path)

        config = SaranCLI(**data)
        cli_group = config.to_group()
        sys.argv = sys.argv[1:]
        cli_group()

    except FileNotFoundError:
        print(f"Error: File '{input_file}' not found.", file=sys.stderr)
        sys.exit(1)
    except ValidationError as e:
        print(f"Error validating configuration: {e}", file=sys.stderr)
        sys.exit(1)

    sys.exit(0)
