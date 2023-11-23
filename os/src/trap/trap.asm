.altmacro
.macro SAVE_GP n
    sd x\n,\n*8(sp) #将第n个寄存器保存到（sp+8*n）的位置，例如SAVE_GP 0，将x0保存到sp+0x8的位置
.endm
.macro LOAD_GP n
    ld x\n,\n*8(sp) #从内存中读取第n个寄存器的值，例如LOAD_GP 1，将sp+1x8的值读取到x0中
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2 #对齐到2的幂次方，因为每条指令的长度为4字节，所以这里对齐到4字节
__alltraps:
    csrrw sp,sscratch,sp #交换sp和sscratch的值，sscratch是一个临时寄存器，用于保存sp的值
    addi sp,sp,-34*8 #将sp减去34*8，用于29个通用寄存器和sepc、sstatus、sscratch寄存器的保存，多余空间预留
    sd x1,1*8(sp) #将x1保存到sp+0x8的位置
    sd x3,3*8(sp) #将x3保存到sp+0x18的位置
    .set n,5 #保存x5-x31
    .rept 27
        SAVE_GP %n
        .set n,n+1
    .endr
    csrr t0,sstatus #将sstatus寄存器的值保存到t0中
    csrr t1,sepc #将sepc寄存器的值保存到t1中,sepc指向trap指令的下一条指令
    sd t0,32*8(sp) #将t0保存到sp+0x80的位置
    sd t1,33*8(sp) #将t1保存到sp+0x88的位置
    csrr t2,sscratch #将sscratch寄存器的值保存到t2中
    sd t2,2*8(sp) #将t2保存到sp+0x10的位置
    mv a0,sp #将sp的值保存到a0中
    call trap_handler #调用trap_handler函数



__restore:
    mv sp,a0 #将a0的值保存到sp中
    ld t0,32*8(sp) #将sp+0x80的值读取到t0中
    ld t1,33*8(sp) #将sp+0x88的值读取到t1中
    ld t2,2*8(sp) #将sp+0x10的值读取到t2中
    csrw sstatus,t0 #将t0的值保存到sstatus寄存器中
    csrw sepc,t1 #将t1的值保存到sepc寄存器中
    csrw sscratch,t2 #将t2的值保存到sscratch寄存器中
    ld x1,1*8(sp) #将sp+0x8的值读取到x1中
    ld x3,3*8(sp) #将sp+0x18的值读取到x3中
    .set n,5 #读取x5-x31
    .rept 27
        LOAD_GP %n
        .set n,n+1
    .endr
    addi sp,sp,34*8 #将sp加上34*8，用于29个通用寄存器和sepc、sstatus、sscratch寄存器的恢复
    csrrw sp,sscratch,sp #交换sp和sscratch的值，sscratch是一个临时寄存器，用于保存sp的值
    sret

