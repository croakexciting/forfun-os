use xmas_elf::ElfFile;

pub fn parse(data: &[u8]) -> Result<ElfFile, &'static str> {
    let elf = ElfFile::new(data)?;
    let header = elf.header;
    let magic = header.pt1.magic;
    if magic != [0x7f, 0x45, 0x4c, 0x46] {
        return Err("Magic number invalid");
    }

    Ok(elf)
}