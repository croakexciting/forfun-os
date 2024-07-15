use alloc::{
    sync::Arc, vec::Vec, vec
};
use spin::mutex::Mutex;

use super::basic::VirtPage;

#[derive(Clone)]
pub struct BuddyAllocator {
    max_order: usize,
    free_list: Vec<Option<Arc<Mutex<BuddyBlock>>>>
}

impl BuddyAllocator {
    pub fn new(max_order: usize, start: VirtPage, pn: usize) -> Self {
        let mut free_list = vec![None; max_order + 1];
        let max_block_pn: usize = 1 << max_order;
        let mut start_vpn = start.0;
        let mut total_vpn = pn;
        while total_vpn >= max_block_pn {
            let block_ptr: Arc<Mutex<BuddyBlock>> = Arc::new(Mutex::new(BuddyBlock::new(start_vpn)));
            block_ptr.lock().next = free_list[max_order].clone();
            free_list[max_order] = Some(block_ptr);
            start_vpn += max_block_pn;
            total_vpn -= max_block_pn;
        }
        Self { max_order, free_list }
    }

    pub fn alloc(&mut self, pn: usize) -> Option<VirtPage> {
        let mut order = 0;
        while (1 << order) < pn {
            order += 1;
        }

        if order > self.max_order {
            println!("[kernel] Request block size is too big");
            return None;
        }

        for i in order..(self.max_order + 1) {
            if let Some(block) = self.free_list[i].clone() {
                self.free_list[i] = block.lock().next.clone();

                let mut o = i;
                while o > order {
                    o -= 1;
                    let split_start_vpn = block.lock().start_vpn + (1 << o);
                    let split_block: Arc<Mutex<BuddyBlock>> = Arc::new(Mutex::new(BuddyBlock::new(split_start_vpn)));
                    split_block.lock().next = self.free_list[o].clone();
                    self.free_list[o] = Some(split_block);
                }
                return Some(block.lock().start_vpn.into())
            }
        }

        return None;
    }

    pub fn dealloc(&mut self, vpn: VirtPage, pn: usize) {
        let mut order: usize = 0;
        while (1 << order) < pn {
            order += 1;
        }

        let block: Arc<Mutex<BuddyBlock>> = Arc::new(Mutex::new(BuddyBlock::new(vpn.0)));

        // 尝试合并到大块
        while order < self.max_order {
            // 异或计算 buddy address，非常非常巧妙
            let buddy_start_vpn = block.lock().start_vpn ^ (1 << order);
            let mut find_buddy_flag = false;

            if let Some(head) = self.free_list[order].clone() {
                if head.lock().start_vpn == buddy_start_vpn {
                    find_buddy_flag = true;
                    self.free_list[order] = head.lock().next.clone();
                } else {
                    let mut current = head.clone();

                    while let Some(next) = current.clone().lock().next.clone() {
                        if next.lock().start_vpn == buddy_start_vpn {
                            find_buddy_flag = true;
                            current.lock().next = next.lock().next.clone();
                            if block.lock().start_vpn > next.lock().start_vpn {
                                block.lock().start_vpn = next.lock().start_vpn;
                            }
                            break;
                        } else {
                            current = next;
                        }
                    }
                }
            }

            if find_buddy_flag == true {
                order += 1;
            } else {
                // 在这一级中已经找不到伙伴，那么再往上也不会有，加入当前链表
                block.lock().next = self.free_list[order].clone();
                self.free_list[order] = Some(block);
                return;
            }
        }
    }
}

pub struct BuddyBlock {
    start_vpn: usize,
    next: Option<Arc<Mutex<Self>>>,
}

impl BuddyBlock {
    pub fn new(start: usize) -> Self {
        Self { start_vpn: start, next: None }
    }
}