
#[derive(Clone, Copy, PartialEq)]
pub enum Reg {
    Zero,
    Arg(usize),
    Gp(usize),
    Ret(usize)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Instruction {
    Load(Reg,usize),
    Add(Reg, Reg, Reg),
    Sub(Reg, Reg, Reg),
    Mul(Reg, Reg, Reg),
    Div(Reg, Reg, Reg),
    Mov(Reg, Reg),
    Cmp(Reg, Reg, Reg),
    And(Reg, Reg, Reg),
    Or(Reg, Reg, Reg),
    CmpBr(Reg,Reg,Reg),
    Jmp(Reg),
    Ret,
}

pub fn exec(_ins: &[Instruction], _regs: &mut [usize]) -> usize {
    panic!()
    // use Instruction::*;

    // let ret_reg = 0;
    // let mut pc = 0;

    // loop {
    //     let i = &ins[pc];

    //     match *i {
    //         Ret => break,
    //         Add(d, a, b) => regs[d] = regs[a] + regs[b],
    //         Sub(d, a, b) => regs[d] = regs[a] - regs[b],
    //         Mul(d, a, b) => regs[d] = regs[a] * regs[b],
    //         Div(d, a, b) => regs[d] = regs[a] / regs[b],
    //         Mov(d, a) => regs[d] = regs[a],
    //         Cmp(d, a, b) => regs[d] = if regs[a] == regs[b] { 1 } else { 0 },
    //         And(d, a, b) => regs[d] = if regs[a] != 0  && regs[b] != 0 {1} else {0},
    //         Or(d, a, b) => regs[d] = if regs[a] != 0  || regs[b] != 0 {1} else {0},
    //         _ => panic!(),
    //     }
    //     pc += 1;
    // }

    // regs[ret_reg]
}

