all:
	dtc -I dts -O dtb -o gameslab_led.dtbo -@ gameslab_led.dtso
	bootgen -image gameslab_led.bif -w -process_bitstream bin
