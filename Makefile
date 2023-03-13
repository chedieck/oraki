install:
	cargo build
	cp target/debug/oraki ${HOME}/.local/bin/oraki
	mkdir -p ${HOME}/.local/share/oraki
	cp extra/main.css ${HOME}/.local/share/oraki/main.css

uninstall:
	rm ${HOME}/.local/bin/oraki
	rm -rf ${HOME}/.local/share/oraki
