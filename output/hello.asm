  .text
  .globl main
main:
  addi sp, sp, -16
.Lentry_2:
  j .L3
.L3:
  li t0, 0
  sw t0, 8(sp)
  j .Lend_1
.L0:
  j .Lend_1
.Lend_1:
  lw t0, 8(sp)
  sw t0, 12(sp)
  lw a0, 12(sp)
  addi sp, sp, 16
  ret
