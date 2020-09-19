use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::alphanumeric1,
    combinator::map,
    multi::{many0, separated_list},
    number::complete::be_i64,
    sequence::{preceded, terminated},
    IResult,
};
use std::fs::File;
use std::io::prelude::*;
#[derive(Debug)]
enum Ops {
    ADD,
    LD,
    STORE,
}

impl Ops {
    fn opcode(&self) -> u16 {
        match self {
            Ops::ADD => 0b0001,
            Ops::LD => 0b0010,
            Ops::STORE => 0b0011,
        }
    }
}

#[derive(Debug, PartialEq)]
enum OType {
    IMM,
    REG,
}

#[derive(Debug, PartialEq)]
struct Operand {
    name: String,
    o_type: OType,
}

#[derive(Debug)]
struct Instruction {
    operation: Ops,
    operand_list: Vec<Operand>,
    o_type: OType,
}

fn reg_to_binary(o: &Operand) -> u16 {
    match &o.name[..] {
        "r1" => 0b001,
        "r2" => 0b010,
        "r3" => 0b011,
        "r4" => 0b100,
        "r5" => 0b101,
        _ => o.name.parse::<u16>().unwrap(),
    }
}

fn parse_op(i: &[u8]) -> IResult<&[u8], Ops> {
    alt((
        map(tag("add"), |_| Ops::ADD),
        map(tag("ld"), |_| Ops::LD),
        map(tag("store"), |_| Ops::STORE),
    ))(i)
}

fn parse_operand(i: &[u8]) -> IResult<&[u8], Operand> {
    // let (out, operand) =
    alt((
        map(alphanumeric1, |s: &[u8]| Operand {
            name: std::str::from_utf8(s).unwrap().to_owned(),
            o_type: OType::REG,
        }),
        map(preceded(tag("$"), alphanumeric1), |s: &[u8]| Operand {
            name: std::str::from_utf8(s).unwrap().to_owned(),
            o_type: OType::IMM,
        }),
    ))(i)
    // Ok((out, std::str::from_utf8(operand).unwrap()))
    // Ok((out, operand))
}

fn parse_instruction(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (input1, op) = parse_op(i)?;
    let (input2, _) = tag(" ")(input1)?;
    let after_op = terminated(separated_list(tag(" "), parse_operand), tag(";"));
    let (input, operand_list) = after_op(input2)?;
    let mut ret = Instruction {
        operation: op,
        operand_list: Vec::new(),
        o_type: OType::REG,
    };
    for operand in operand_list {
        // ret.operand_list.push(operand.to_owned());
        if operand.o_type == OType::IMM {
            ret.o_type = OType::IMM;
        }
        ret.operand_list.push(operand);
    }
    Ok((input, ret))
}

fn parse_program(i: &[u8]) -> IResult<&[u8], Vec<Instruction>> {
    many0(parse_instruction)(i)
}

fn add_imm_bits(i: &Instruction) -> u16 {
    Ops::opcode(&i.operation) << 12
        | reg_to_binary(&i.operand_list[0]) << 9
        | reg_to_binary(&i.operand_list[1]) << 6
        | 0b1 << 5
        | reg_to_binary(&i.operand_list[2])
}

fn add_reg_bits(i: &Instruction) -> u16 {
    Ops::opcode(&i.operation) << 12
        | reg_to_binary(&i.operand_list[0]) << 9
        | reg_to_binary(&i.operand_list[1]) << 6
        | 0b000 << 3
        | reg_to_binary(&i.operand_list[2])
}

// TODO: handle whitespace between instructions

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inst = b"add r1 r2 $4;add r2 r3 r5;";
    let (_, parsed_program) = parse_program(&inst[..]).unwrap();
    println!("{:?}", parsed_program);
    let mut bytes: Vec<u8> = Vec::new();
    // let mut inst_bin: i16;
    for i in parsed_program {
        // bytes.push(Ops::to_binary(i.operation));
        let inst_bits: u16 = match i.o_type {
            OType::IMM => add_imm_bits(&i),
            OType::REG => add_reg_bits(&i),
        };
        println!("inst_bits: {:#018b}", inst_bits);
        bytes.extend_from_slice(&(inst_bits).to_be_bytes());
    }
    println!("instruction: {:#b}", bytes[0]);
    {
        let mut file = File::create("out")?;
        file.write_all(&bytes[..])?;
    }

    {
        let mut file = File::open("out")?;
        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer)?;
        println!("{:?}", buffer);
    }

    Ok(())
}
