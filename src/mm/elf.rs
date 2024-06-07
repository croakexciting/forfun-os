use xmas_elf::ElfFile;

pub fn parse(data: &[u8]) -> ElfFile {
    let elf = ElfFile::new(data).unwrap();
    
}