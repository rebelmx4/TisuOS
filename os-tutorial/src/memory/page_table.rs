//! # page table
//! 负责管理 satp 及对应的页表项，页表模式暂定为 RV39
//! > 页表映射部分经过检查应该没有问题
//! 2020年12月13日 zg

/// ## SATP
pub struct SATP{
    pub flag : usize,
}
impl SATP {
    pub fn new(ppn : usize, asid : usize, mode : usize) -> Self{
        SATP{
            flag : ((ppn >> 12) & 0xfff_ffff_ffff) | ((asid & 0xffff) << 44) | ((mode & 0xf) << 60)
        }
    }
    pub fn from(satp : usize)->Self{
        SATP{
            flag : satp
        }
    }
    pub fn get_ppn_addr(&self)->usize{
        (self.flag & 0xfff_ffff_ffff) << 12
    }
    pub fn val(&self) -> usize{
        self.flag as usize
    }
    pub fn get_page_table(&self) ->*mut PageTable{
        self.get_ppn_addr() as *mut PageTable
    }
    pub fn is_map(&self)->bool{
        self.get_ppn_addr() != 0
    }
    pub fn create_page_table(&mut self){
        let pt = PageTable::new();
        assert!(!pt.is_null());
        let ppn = pt as usize;
        self.flag = ((ppn >> 12) & 0xfff_ffff_ffff) | ((MODE & 0xf) << 60);
    }
    pub fn free_page_table(&self){
        if self.is_map(){
            let pt = self.get_ppn_addr() as *mut PageTable;
            unsafe {
                (*pt).free();
            }
        }
    }
}
/// ## PageBit
/// 页表属性，与 page.rs 中的 PageBit 无关
#[allow(dead_code)]
pub enum PageBit{
    Valid   = 1 << 0,
    Read    = 1 << 1,
    Write   = 1 << 2,
    Excute  = 1 << 3,
    User    = 1 << 4,
    Global  = 1 << 5,
    Access  = 1 << 6,
    Dirty   = 1 << 7,
}
impl PageBit {
    pub fn val(self) -> u64 {
        self as u64
    }
}
/// ## PTE
/// 页表项的 PPN 给出下一级页表地址或者实际的物理地址
/// 以一个页表为单位，所以给出的地址需要先右移 12 位
#[derive(Copy, Clone)]
pub struct PTE{
    flag : u64
}

#[allow(dead_code)]
impl PTE {
    pub fn new() -> Self{
        PTE{
            flag : 0
        }
    }
    pub fn is_valid(&self) -> bool {
        self.flag & PageBit::Valid.val() != 0
    }
    pub fn set_node_ppn(&mut self, ppn : u64){
        self.flag = (self.flag & 0x3ff) | ((ppn >> 12) << 10);
    }
    pub fn set_leaf_ppn(&mut self, ppn : u64){
        self.flag = (self.flag & 0x3ff) | ((ppn >> 12) << 10);
    }
    pub fn set_user(&mut self){
        self.flag |= PageBit::User.val();
    }
    pub fn get_ppn(&self) -> usize {
        ((self.flag & !0x3ff) << 2) as usize
    }
    pub fn set_flag(&mut self, flag : u64){
        self.flag |= flag;
    }
    pub fn is_leaf(&self) -> bool {
        self.flag & (PageBit::Read.val() | PageBit::Write.val() | PageBit::Excute.val()) != 0
    }
    pub fn set_valid(&mut self){
        self.flag |= PageBit::Valid.val();
    }
}
/// ## PageTable
/// Rv39 页表，本身占据一个页表（默认4KB）的大小
/// 分为三级，页表地址必须以一个页表的大小对齐
pub struct PageTable{
    entry : [PTE; 512],
}
/// 页表结构
/// 负责在内存中创建、管理对应的映射
#[allow(dead_code)]
impl PageTable {
    pub fn new() -> *mut Self{
        let addr = page::alloc_kernel_page(1);
        addr as *mut Self
    }
    pub fn map_user(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Write.val()
            | PageBit::Excute.val() | PageBit::User.val());
    }
    pub fn map_kernel(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Write.val()
            | PageBit::Excute.val());
    }
    pub fn map_kernel_data(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Write.val());
    }
    pub fn map_kernel_code(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Excute.val());
    }
    pub fn map_user_data(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Write.val() |
            PageBit::User.val());
    }
    pub fn map_user_code(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::User.val()
        | PageBit::Excute.val());
    }
    pub fn map(&mut self, virtual_addr : usize, physic_addr : usize, flag : u64){
        let vpn = [
            (virtual_addr >> 30) & 0x1ff,
            (virtual_addr >> 21) & 0x1ff,
            (virtual_addr >> 12) & 0x1ff
        ];
        let pte_first = &mut self.entry[vpn[0]];
        if !pte_first.is_valid() {
            let addr = page::alloc_kernel_page(1);
            assert!(!addr.is_null());
            pte_first.set_node_ppn(addr as u64);
            pte_first.set_flag(flag & 
                !PageBit::Read.val() & !PageBit::Write.val() & !PageBit::Excute.val());
            pte_first.set_valid();
        }
        let table_mid = unsafe {&mut *(pte_first.get_ppn() as *mut Self)};
        let pte_mid = &mut table_mid.entry[vpn[1]];
        
        if !pte_mid.is_valid() {
            let addr = page::alloc_kernel_page(1);
            assert!(!addr.is_null());
            pte_mid.set_node_ppn(addr as u64);
            pte_mid.set_flag(flag &
                !PageBit::Read.val() & !PageBit::Write.val() & !PageBit::Excute.val());
            pte_mid.set_valid();
        }
        let table_final = unsafe {&mut *(pte_mid.get_ppn() as *mut Self)};
        let pte_final = &mut table_final.entry[vpn[2]];
        pte_final.flag = 0;
        pte_final.set_leaf_ppn(physic_addr as u64);
        pte_final.set_flag(flag);
        pte_final.set_valid();
    }
    pub fn unmap(&mut self, virtual_addr : usize){
        let vpn = [
            (virtual_addr >> 30) & 0x1ff,
            (virtual_addr >> 21) & 0x1ff,
            (virtual_addr >> 12) & 0x1ff
        ];
        let pte_first = &mut self.entry[vpn[0]];
        assert!(pte_first.is_valid());
        let table_mid = unsafe {&mut *(pte_first.get_ppn() as *mut Self)};
        let pte_mid = &mut table_mid.entry[vpn[1]];
        assert!(pte_mid.is_valid());

        let table_final = unsafe {&mut *(pte_mid.get_ppn() as *mut Self)};
        let pte_final = &mut table_final.entry[vpn[2]];
        assert!(pte_final.is_valid() && pte_final.is_leaf());
    }
    pub fn free(&mut self){
        for i in 0..512{
            let pte = &self.entry[i];
            if pte.is_valid(){
                let next_table = unsafe{&*(pte.get_ppn() as *mut PageTable)};
                for j in 0..512{
                    let pte = next_table.entry[j];
                    if pte.is_valid(){
                        let next_table = unsafe{&*(pte.get_ppn() as *mut PageTable)};
                        for k in 0..512{
                            let pte = next_table.entry[k];
                            if pte.is_valid(){
                                page::free_page(pte.get_ppn() as *mut u8);
                            }
                        }
                        page::free_page(pte.get_ppn() as *mut u8);
                    }
                }
                page::free_page(pte.get_ppn() as *mut u8);
            }
        }
        let addr = self as *mut Self;

        page::free_page(addr as *mut u8);
    }
    pub fn print(&mut self){
        println!("table at {:x}", ((&mut *self) as *mut Self) as usize);
        for i in 0..512{
            let pte = self.entry[i];
            if pte.is_valid(){
                println!("{}", i);
                if pte.is_leaf(){
                    println!("leaf {}, physic address {:08x}", i, pte.get_ppn());
                }
                else {
                    let next_table = pte.get_ppn() as *mut PageTable;
                    unsafe{
                        (*next_table).print();
                    }
                }
            }
        }
    }
}

static MODE : usize = 8;

pub fn make_satp(pt : usize, asid : usize) -> usize{
    SATP::new(pt, asid, MODE).val()
}


/// ## 将内核代码部分进行映射
/// 因为进程返回调用了内核中的代码
pub fn map_kernel_area(pt : &mut PageTable, is_kernel : bool){
    unsafe {
        if is_kernel{
            let st = page::MEMORY_START;
            let ed = page::MEMORY_END;
            for addr in (st..ed).step_by(page::PAGE_SIZE){
                pt.map_kernel(addr, addr);
            }
            for i in 0..10{
                let addr =0x1000_0000 + i * page::PAGE_SIZE;
                pt.map_kernel(addr,addr);
            }
            let addr = 0x200_0000;
            for i in 0..16 {
                let addr = addr + i * page::PAGE_SIZE;
                pt.map_kernel_data(addr,addr);
            }
        }
        else{
            let st = page::MEMORY_START;
            let ed = page::KERNEL_HEAP_START;
            for addr in (st..ed).step_by(page::PAGE_SIZE){
                pt.map_user_code(addr, addr);
            }
        }
    }
}

use crate::uart;
use crate::page;