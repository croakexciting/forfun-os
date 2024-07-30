use fdt::{Fdt, node::FdtNode};

pub fn parse_dt(addr: usize) {
    let fd = unsafe {
        Fdt::from_ptr(addr as *const u8).unwrap()
    };

    for node in fd.all_nodes() {
        if let Some(compitable) = node.compatible() {
            for name in compitable.all() {
                println!("node is: {}", name);
            }

            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                println!("addr is : {:#x}", reg.starting_address as usize);
            }

            for p in node.properties() {
                println!("p is {}, {}", p.name, core::str::from_utf8(p.value).unwrap());
            }
        }
    }
}