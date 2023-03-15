install:
	cargo build
	cp target/debug/oraki ${HOME}/.local/bin/oraki
	mkdir -p ${HOME}/.local/share/oraki
	cp extra/style.css ${HOME}/.local/share/oraki/style.css

uninstall:
	rm ${HOME}/.local/bin/oraki
	rm -rf ${HOME}/.local/share/oraki
