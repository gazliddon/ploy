pub type Reg = usize;




#[derive(Clone,Copy,PartialEq)]
pub enum Instructions {
    Add(Reg,Reg,Reg),
    Sub(Reg,Reg,Reg),
    Mul(Reg,Reg,Reg),
    Div(Reg,Reg,Reg),
    Mov(Reg,Reg),
    Ret,
}

pub fn exec(ins : &[Instructions], regs: &mut [usize]) -> usize{
    use Instructions::*;

    let ret_reg = 0;
    let mut pc = 0;

    loop {
        let i = &ins[pc];

        match *i {
            Ret => break,
            Add(d,a,b) => regs[d] = regs[a] + regs[b],
            Sub(d,a,b) => regs[d] = regs[a] - regs[b],
            Mul(d,a,b) => regs[d] = regs[a] * regs[b],
            Div(d,a,b) => regs[d] = regs[a] / regs[b],
            Mov(d,a) => regs[d] = regs[a] ,
        }
        pc = pc + 1;
    }

    regs[ret_reg]
}

pub fn test() {
    use Instructions::*;

    let mut regs = [0;100];

    // ret = a * b + a
    regs[0] = 10;
    regs[1] = 20;

    let func = [
        Mul(2,0,1),
        Add(2,2,1),
        Mov(0,2),
    ];

    exec(&func, &mut regs[..]);

    println!("Result is {}", regs[0]);
}
