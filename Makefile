image:
	bootgen -image gs-linux.bif -arch zynq -w -o gs-linux.bin

program:
	program_flash -verify -f gs-linux.bin -offset 0 -flash_type qspi_single
