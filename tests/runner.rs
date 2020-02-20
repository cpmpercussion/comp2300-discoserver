mod common;
use common::{load_program};

#[test]
fn mov() {
    let mut board = load_program("mov").unwrap();
    let orig_flags = board.cpu.get_flags();

    board.step().unwrap();
    assert_eq!(board.read_reg(0u32), 0);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, true);
    assert_eq!(flags.c, orig_flags.c);

    board.step().unwrap();
    assert_eq!(board.read_reg(0u32), 5);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, false);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, orig_flags.c);

    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), 0b11001010 << 24);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, true);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, true);

    board.step().unwrap();
    assert_eq!(board.read_reg(1u32), 0b11001010 << 23);
    let flags = board.cpu.get_flags();
    assert_eq!(flags.n, true);
    assert_eq!(flags.z, false);
    assert_eq!(flags.c, true);


    board.step().unwrap();
    assert_eq!(board.read_reg(2u32), 65535);
}
