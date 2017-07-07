@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@ Constants

.equ C_GPIO_BASE,       0xE000A000
.equ C_GPIO_DIRM_0,     0x00000204
.equ C_GPIO_OEN_0,      0x00000208
.equ C_GPIO_DATA_0,     0x00000040
.equ C_SLCR_BASE,       0xF8000000
.equ C_SLCR_UNLOCK,     0x00000008
.equ C_SLCR_LOCK,       0x00000004
.equ C_SLCR_MIO_PIN_07, 0x0000071C
.equ C_SLCR_LOCK_KEY,   0x767B
.equ C_SLCR_UNLOCK_KEY, 0xDF0D

.equ C_DELAY,           0x00400000

@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@ Vector table

.section .text
.code 32
.globl vectors
vectors:
	b entry				@ reset
	b .					@ undefined instruction
	b .					@ software interrupt
	b .					@ prefetch abort
	b .					@ data abort
	b .					@ hypervisor entry
    b .                 @ interrupt
	b .					@ fast interrupt



entry:
    @ unlock SLCR
    ldr r0, SLCR_BASE
    ldr r1, SLCR_UNLOCK_KEY
    str r1, [r0, #C_SLCR_UNLOCK]

    @ setup MIO pin, LVCMO33 and no tri-state
    mov r1, #0x600
    str r1, [r0, #C_SLCR_MIO_PIN_07]

    @ lock SLCR
    ldr r1, SLCR_LOCK_KEY
    str r1, [r0, #C_SLCR_LOCK]

    @ setup GPIO dir, output en, and data
    ldr r0, GPIO_BASE
    mov r1, #0x80
    mov r2, #0x0
    str r1, [r0, #C_GPIO_DIRM_0]
    str r1, [r0, #C_GPIO_OEN_0]
    str r1, [r0, #C_GPIO_DATA_0]

loop_outer:
    mov r3, #C_DELAY
loop0:
    SUBS r3, r3, #1
    BNE loop0

    str r2, [r0, #C_GPIO_DATA_0]

    mov r3, #C_DELAY
loop1:
    SUBS r3, r3, #1
    BNE loop1

    str r1, [r0, #C_GPIO_DATA_0]
    b loop_outer
    

    b .
    
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@ Literal pool
GPIO_BASE:       .word C_GPIO_BASE
SLCR_BASE:       .word C_SLCR_BASE
SLCR_LOCK_KEY:   .word C_SLCR_LOCK_KEY
SLCR_UNLOCK_KEY: .word C_SLCR_UNLOCK_KEY
