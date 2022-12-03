all:
	python3 run.py

debug:
	python3 run.py slow.asm debug

clean:
	cd j_system/j_assembler; cargo clean
	cd j_system/j_interpreter; cargo clean
	cd j_system/j_system_definition/; cargo clean
check:
	cd j_system/j_assembler; cargo check
	cd j_system/j_interpreter; cargo check
	cd j_system/j_system_definition/; cargo check
