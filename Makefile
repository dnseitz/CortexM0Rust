binary = altos_rust
kernel = target/$(target)/debug/$(binary)
linker_script = rust.ld
target = cortex-m0


gdb_flags = 
st_port = 4242
ocd_port = 3333
st_gdb_flags = $(gdb_flags) -eval-command="target remote :$(st_port)"
ocd_gdb_flags = $(gdb_flags) -eval-command="target remote :$(ocd_port)"

all: cargo

cargo: $(linker_script)
	@xargo build --target $(target)

gdb: cargo
	@arm-none-eabi-gdb $(gdb_flags) $(kernel)
	
gdb-st: cargo
	@arm-none-eabi-gdb $(st_gdb_flags) $(kernel)

gdb-ocd: cargo
	@arm-none-eabi-gdb $(ocd_gdb_flags) $(kernel)
