use core::fmt::Debug;

use super::page_table::PageTableEntry;

use crate::config::*;
const PA_WIDTH_SV39: usize = 56;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct PhysAddr(pub usize);
#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq,Debug)]
pub struct VirtAddr(pub usize);
#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct PhysPageNum(pub usize);
#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq,Debug)]
pub struct VirtPageNum(pub usize);
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PA_WIDTH_SV39) - 1))
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
impl From<VirtAddr> for VirtPageNum{
    fn from(v:VirtAddr)->Self{
        Self(v.0>>PAGE_SIZE_BITS)
    }
}

impl From<VirtAddr> for usize{
    fn from(v: VirtAddr) -> Self {
        v.0
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}
impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        VirtAddr(v)
    }
}


impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}


impl PhysAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
    pub fn get_mut<T>(&self)->&'static mut T{
        unsafe{(self.0 as *mut T).as_mut().unwrap()}
    }
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }
}

impl VirtAddr{
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}

impl PhysPageNum {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE) }
    }
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, PAGE_SIZE / 8) }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = self.clone().into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}
impl StepByOne for PhysPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }

}

impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

pub trait StepByOne{
    fn step(&mut self);
}

impl StepByOne for VirtPageNum{
    fn step(&mut self){
        self.0+=1;
    }
}

#[derive(Copy, Clone)]
pub struct SimpleRange<T>
where T:StepByOne+Copy+PartialEq+PartialOrd+Debug
{
    l:T,
    r:T
}

impl<T>SimpleRange<T>
where T:StepByOne+Copy+PartialEq+PartialOrd+Debug
{
    pub fn new(l:T,r:T)->Self{
        assert!(l<=r,"l={:?} should be less than or equal to r={:?}",l,r);
        Self{l,r}
    }
    pub fn get_start(&self)->T{
        self.l
    }
    pub fn get_end(&self)->T{
        self.r
    }
}

impl<T> IntoIterator for SimpleRange<T>
where T:StepByOne+Copy+PartialEq+PartialOrd+Debug
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l,self.r)
    }
}

pub struct SimpleRangeIterator<T>
where T:StepByOne+Copy+PartialEq+PartialOrd+Debug
{
    current:T,
    end:T,
}

impl<T> SimpleRangeIterator<T>
where T:StepByOne+Copy+PartialEq+PartialOrd+Debug
{
    pub fn new(l:T,r:T)->Self{
        Self{current:l,end:r}
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where T:StepByOne+Copy+PartialEq+PartialOrd+Debug
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current==self.end{
            None
        }else{
            let result=self.current;
            self.current.step();
            Some(result)
        }
    }
}

pub type VPNRange=SimpleRange<VirtPageNum>;