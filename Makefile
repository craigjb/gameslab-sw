image:
	bootgen -image gs-linux.bif -arch zynq -w -o gs-linux.bin

program:
	program_flash -f gs-linux.bin -verify -offset 0 -flash_type qspi_single
