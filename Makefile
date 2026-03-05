install:
	uv sync

deploy: install
	sudo cp -r .venv/bin/saran /usr/local/bin/saran

uninstall:
	sudo rm -f /usr/local/bin/saran
