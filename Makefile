install:
	uv sync

deploy: install
	sudo cp -r .venv/bin/grip /usr/local/bin/grip

uninstall:
	sudo rm -f /usr/local/bin/grip
