use crate::MemError;
use crate::peripherals::Peripheral;
use crate::utils::io::read_register;
use crate::utils::bits::bitset;

#[derive(Debug)]
struct Cr {
    pllrdy: bool,
    pllon: bool,
    pllsai1rdy: bool,
    pllsai1on: bool,
    pllsai2rdy: bool,
    pllsai2on: bool,

    csson: bool,
    hsebyp: bool,
    hserdy: bool,
    hseon: bool,
    hsiasfs: bool,
    hsirdy: bool,
    hsikeron: bool,
    hsion: bool,

    msirange: u8,
    msirgsel: bool,
    msipllen: bool,
    msirdy: bool,
    msion: bool,
}

impl Cr {
    fn new() -> Cr {
        return Cr {
            pllrdy: false,
            pllon: false,
            pllsai1rdy: false,
            pllsai1on: false,
            pllsai2rdy: false,
            pllsai2on: false,
            csson: false,
            hsebyp: false,
            hserdy: false,
            hseon: false,
            hsiasfs: false,
            hsirdy: false,
            hsikeron: false,
            hsion: false,
            msirange: 0b0110,
            msirgsel: false,
            msipllen: false,
            msirdy: true,
            msion: true,
        };
    }

    fn reset(&mut self) {
        self.pllrdy = false;
        self.pllon = false;
        self.pllsai1rdy = false;
        self.pllsai1on = false;
        self.pllsai2rdy = false;
        self.pllsai2on = false;
        self.csson = false;
        // self.hsebyp unchanged;
        self.hserdy = false;
        self.hseon = false;
        self.hsiasfs = false;
        self.hsirdy = false;
        self.hsikeron = false;
        self.hsion = false;
        self.msirange = 0b0110;
        self.msirgsel = false;
        self.msipllen = false;
        self.msirdy = true;
        self.msion = true;
    }

    fn read(&self) -> u32 {
        let result: u32 =
            (self.pllsai2rdy as u32) << 29 |
            (self.pllsai2on as u32) << 28 |
            (self.pllsai1rdy as u32) << 27 |
            (self.pllsai1on as u32) << 26 |
            (self.pllrdy as u32) << 25 |
            (self.pllon as u32) << 24 |
            (self.csson as u32) << 19 |
            (self.hsebyp as u32) << 18 |
            (self.hserdy as u32) << 17 |
            (self.hseon as u32) << 16 |
            (self.hsiasfs as u32) << 11 |
            (self.hsirdy as u32) << 10 |
            (self.hsikeron as u32) << 9 |
            (self.hsion as u32) << 8 |
            (self.msirange as u32) << 4 |
            (self.msirgsel as u32) << 3 |
            (self.msipllen as u32) << 2 |
            (self.msirdy as u32) << 1 |
            (self.msion as u32) << 0;
        return result;
    }

    fn write(&mut self, val: u32) {
        let pllsai2on = bitset(val, 28);
        self.pllsai2on = pllsai2on;
        self.pllsai2rdy = pllsai2on; // assume instant lock / unlock

        let pllsai1on = bitset(val, 26);
        self.pllsai1on = pllsai1on;
        self.pllsai1rdy = pllsai1on; // assume instant lock / unlock

        let pllon = bitset(val, 24);
        self.pllon = pllon;
        self.pllrdy = pllon; // assume instant lock / unlock

        self.csson = bitset(val, 19);

        if !self.hseon {
            self.hsebyp = bitset(val, 18);
        }

        let hseon = bitset(val, 16);
        self.hserdy = hseon; // TODO: meant to go off after 6 clock cycles
        self.hseon = hseon;

        self.hsiasfs = bitset(val, 11);

        let hsion = bitset(val, 8);
        self.hsirdy = hsion; // assume instant stable / off
        self.hsion = hsion;

        self.hsikeron = bitset(val, 9);

        let msirange: u8 = ((val >> 4) & 0xF) as u8;
        if msirange != self.msirange {
            if !self.msion || self.msirdy { // always true when we assume instant locks
                self.msirange = msirange;
            } else {
                println!("Cannot edit MSI range while MSI on and MSI not ready");
            }
        }

        self.msirgsel = self.msirgsel || bitset(val, 3); // write 0 has no effect

        self.msipllen = bitset(val, 2); // TODO: Cannot enable when LSE not ready. Cleared when LSE disabled

        let msion = bitset(val, 0);
        self.msirdy = msion;
        self.msion = msion;
    }
}

#[derive(Debug)]
pub struct RCC {
    cr: Cr,
    icscr: u32,
    cfgr: u32,
    pllcfgr: u32,
    pllsai1cfgr: u32,
    pllsai2cfgr: u32,
    cier: u32,
    cifr: u32,
    cicr: u32,
    ahb1rstr: u32,
    ahb2rstr: u32,
    ahb3rstr: u32,
    apb1rstr1: u32,
    apb1rstr2: u32,
    apb2rstr: u32,
    ahb1enr: u32,
    ahb2enr: u32,
    ahb3enr: u32,
    apb1enr1: u32,
    apb1enr2: u32,
    apb2enr: u32,
    ahb1smenr: u32,
    ahb2smenr: u32,
    ahb3smenr: u32,
    apb1smenr1: u32,
    apb1smenr2: u32,
    apb2smenr: u32,
    ccipr: u32,
    bdcr: u32,
    csr: u32,
}

impl RCC {
    fn new() -> RCC {
        return RCC {
            cr: Cr::new(),
            icscr: 0x1071_0096,
            cfgr: 0x0000_0000,
            pllcfgr: 0x0000_1000,
            pllsai1cfgr: 0x0000_1000,
            pllsai2cfgr: 0x0000_1000,
            cier: 0x0000_0000,
            cifr: 0x0000_0000,
            cicr: 0x0000_0000,
            ahb1rstr: 0x0000_0000,
            ahb2rstr: 0x0000_0000,
            ahb3rstr: 0x0000_0000,
            apb1rstr1: 0x0000_0000,
            apb1rstr2: 0x0000_0000,
            apb2rstr: 0x0000_0000,
            ahb1enr: 0x0000_0100,
            ahb2enr: 0x0000_0000,
            ahb3enr: 0x0000_0000,
            apb1enr1: 0x0000_0000,
            apb1enr2: 0x0000_0000,
            apb2enr: 0x0000_0000,
            ahb1smenr: 0x0001_1303,
            ahb2smenr: 0x0005_32FF,
            ahb3smenr: 0x0000_0101,
            apb1smenr1: 0xF2FE_CA3F,
            apb1smenr2: 0x0000_0025,
            apb2smenr: 0x0167_7C01,
            ccipr: 0x0000_0000,
            bdcr: 0x0000_0000,
            csr: 0x0C00_0600,
        }
    }
}

impl Default for RCC {
    fn default() -> RCC {
        return RCC::new();
    }
}

impl Peripheral for RCC {
    fn read(&self, offset: u32, size: usize) -> Result<u32, MemError> {
        return match offset {
            0x00..=0x03 => Ok(self.cr.read()),
            0x04..=0x07 => read_register(self.icscr, offset - 0x04, size),
            0x08..=0x0B => read_register(self.cfgr, offset - 0x08, size),
            0x0C..=0x0F => read_register(self.pllcfgr, offset - 0x0C, size),
            0x10..=0x13 => read_register(self.pllsai1cfgr, offset - 0x10, size),
            0x14..=0x17 => read_register(self.pllsai2cfgr, offset - 0x14, size),
            0x18..=0x1B => read_register(self.cier, offset - 0x18, size),
            0x1C..=0x1F => read_register(self.cifr, offset - 0x1C, size),
            0x20..=0x23 => read_register(self.ahb1rstr, offset - 0x20, size),
            0x24..=0x27 => read_register(self.ahb2rstr, offset - 0x24, size),
            0x28..=0x2B => read_register(self.ahb3rstr, offset - 0x28, size),
            0x58..=0x5B => read_register(self.apb1enr1, offset - 0x58, size),
            _ => Err(MemError::Unimplemented),
        }

        // println!("Returning {:#010X}", self.icscr);
        // return Ok(self.icscr);
    }

    fn write(&mut self, address: u32, _size: usize) -> Result<(), MemError> {
        println!("RCC write at {:#010X} is unimplemented", address);
        return Ok(());
    }

    fn reset(&mut self) {
        // TODO: Distinguish reset kinds
        self.cr.reset();
        self.icscr = 0x1071_0096;
        self.cfgr = 0x0000_0000;
        self.pllcfgr = 0x0000_1000;
        self.pllsai1cfgr = 0x0000_1000;
        self.pllsai2cfgr = 0x0000_1000;
        self.cier = 0x0000_0000;
        self.cifr = 0x0000_0000;
        self.cicr = 0x0000_0000;
        self.ahb1rstr = 0x0000_0000;
        self.ahb2rstr = 0x0000_0000;
        self.ahb3rstr = 0x0000_0000;
        self.apb1rstr1 = 0x0000_0000;
        self.apb1rstr2 = 0x0000_0000;
        self.apb2rstr = 0x0000_0000;
        self.ahb1enr = 0x0000_0100;
        self.ahb2enr = 0x0000_0000;
        self.ahb3enr = 0x0000_0000;
        self.apb1enr1 = 0x0000_0000;
        self.apb1enr2 = 0x0000_0000;
        self.apb2enr = 0x0000_0000;
        self.ahb1smenr = 0x0001_1303;
        self.ahb2smenr = 0x0005_32FF;
        self.ahb3smenr = 0x0000_0101;
        self.apb1smenr1 = 0xF2FE_CA3F;
        self.apb1smenr2 = 0x0000_0025;
        self.apb2smenr = 0x0167_7C01;
        self.ccipr = 0x0000_0000;
        self.bdcr = 0x0000_0000;
        self.csr = 0x0C00_0600;
    }
}
