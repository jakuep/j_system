use std::cmp::Ordering;

/// The whole memory that is visible to the VM

/*
    **Highest adress**

    Stack
    |
    |
    V


    ʌ
    |
    |
    Heap
    Code
    Rom
    
    **Lowest adress**
*/

const RWX_ROM: bool = true;

struct Mem(Vec<u64>);

impl Mem {
    pub fn new(size: usize) -> Self
    {
        let mut mem = Vec::with_capacity(size);
        // pre-init memory
        (0..size).for_each(|_| mem.push(0));
        Self{0:mem}
    }

    pub fn store(&mut self, adress: u64, value: u64) -> Result<(),String>
    {
        if adress >= self.0.len() as u64
        {
            return Err("this adress is outside of the adress space".into())
        }

        self.0[adress as usize] = value;

        Ok(())
    }

    pub fn read(&self, adress: u64) -> Option<u64>
    {
        if adress >= self.0.len() as u64
        {
            return None
        }

        Some(self.0[adress as usize])
    }
}

pub struct MemModel
{
    mem: Mem,

    /// holds a list sorted all allocations that were made through `malloc`
    ///  
    alloc_tabel: AllocationTable,

    /// the highest possible adress that is able to be used by the heap.
    /// ONLY GETS UPDATED WHEN A SYSCALL HAPPENS.
    heap_cutoff: u64,

    /// rom base pointer
    // TODO: should always be 1, right?
    rom_base_ptr: u64,
    rom_size: u64,

    /// code base pointer
    code_base_ptr: u64,
    code_size: u64,

    /// the maximum of mem that the machine can 
    /// alocate combined. this includes rom, code,
    /// heap and stack
    mem_size:u64,
}

impl MemModel
{

    pub fn new(mem_size: u64) -> Self
    {
        MemModel{ 
            mem: Mem::new(mem_size as usize),
            alloc_tabel: AllocationTable::new(),
            heap_cutoff: mem_size -1,
            rom_base_ptr: 1,
            rom_size: 0,
            code_base_ptr: 0,
            code_size: 0,
            mem_size,
        } 

    }

    /// inserts rom and code into the memory modell
    pub fn prepare_mem(&mut self,rom: Vec<u64>,code:Vec<u64>) 
    {
        // make sure programm and constants fit in memory
        if !(rom.len() + code.len() <= self.mem_size as usize)
        {
            panic!("not enough memory");
        }

        self.rom_base_ptr = 0;
        self.code_base_ptr= self.rom_base_ptr+ rom.len() as u64;

        rom.iter().enumerate().for_each(|(ii,val)| self.mem.store(self.rom_base_ptr + ii as u64, *val).unwrap());
        code.iter().enumerate().for_each(|(ii,val)| self.mem.store(self.code_base_ptr + ii as u64, *val).unwrap());

    }

    pub fn get_heap_cutoff(&self) -> u64
    {
        self.heap_cutoff
    }

    pub fn set_heap_cutoff(&mut self, val: u64)
    {
        self.heap_cutoff = val
    }

    pub fn get_mem_size(&self) -> u64
    {
        self.mem_size
    }

    pub fn store(&mut self,val: u64 ,addr: u64) -> Result<(),String>
    {
        if self.mem_size-1 < addr
        {
            return Err(format!("adress is not in the adressspace?: {}",addr))
        }

        self.mem.store(addr, val)
    }

    /// returns the value at the given adress.
    /// If the given adress adress cant be accessed `None` will be returnd
    pub fn read(&self, addr: u64) -> Result<u64,String>
    {
        if self.mem_size -1 < addr
        {
            return Err("adress is not in the adressspace?".into())
        }

        if addr == 0
        {
            return Err("tried to deref NULL!".into())
        }

        self.mem.read(addr).ok_or("".into())
    }

    pub fn malloc(&mut self, size: u64) -> Option<u64>
    {
        // Do not allow to allocate no mem with malloc
        if size == 0
        {
            return None;
        }

        // use best fit to find the smallest spot that still fits the allocation
        let mut possible_spot_ptr: Option<u64> = None;
        let mut possible_spot_size: Option<u64> = None;

        // last adress of code-section
        let code_end = self.code_base_ptr + self.code_size;

        // if there are no allocations in the allocations-table we can directly check 
        // between the end of the stack and the end of the code section
        if self.alloc_tabel.len() == 0
        {
            if self.heap_cutoff - code_end  >= size
            {
                // allocation adress should be as low as possible to 
                // give as much space to the stack as possible.
                let ptr = code_end + size;

                self.alloc_tabel.insert(Allocation{ptr, size});
                
                // clear memory
                (1+code_end..=code_end+size).for_each(|x| self.mem.store(x, 0).unwrap());

                return Some(ptr);
            }
            else 
            {
                return None;
            }
        }

        // check between first allocation and the end of the code-section
        if self.alloc_tabel.peek(0).ptr - code_end >= size
        {
            // allocation adress should be as low as possible to 
            // give as much space to the stack as possible.
            let ptr = code_end + size;

            possible_spot_ptr = Some(ptr);
            possible_spot_size = Some(self.alloc_tabel.peek(0).ptr - code_end);
        }

        // check between allocations 
        for ii in 1..self.alloc_tabel.len()
        {
            let chunk_size = self.alloc_tabel.peek(ii).ptr - (self.alloc_tabel.peek(ii-1).ptr + self.alloc_tabel.peek(ii-1).size);

            // the chunk must be effectivly smaller to be a better choise for allocating.
            if chunk_size >= size && chunk_size < possible_spot_size.unwrap_or(u64::MAX)
            {
                // allocation adress should be as low as possible to 
                // give as much space to the stack as possible.
                let ptr = self.alloc_tabel.peek(ii-1).ptr + size;
    
                possible_spot_ptr = Some(ptr);
                possible_spot_size = Some(chunk_size);
    
            }
        }

        // check bewtween tos and highest allocation
        
        // highest allocation index 
        let index = self.alloc_tabel.len()-1;
        let chunk_size = self.heap_cutoff - (self.alloc_tabel.peek(index).ptr + self.alloc_tabel.peek(index).size);

        if chunk_size >= size && chunk_size < possible_spot_size.unwrap_or(u64::MAX)
        {
            // allocation adress should be as low as possible to 
            // give as much space to the stack as possible.
            let ptr = self.alloc_tabel.peek(index).ptr + size;

            possible_spot_ptr = Some(ptr);
        }

        // if a allocation is happenening the memory must be nulled?
        if let Some(ptr) = possible_spot_ptr
        {
            (ptr..ptr+size).for_each(|x| self.mem.store(x, 0).unwrap());
        } 
        possible_spot_ptr
    }

    pub fn heap_free(&mut self, ptr: u64)
    {
        // TODO: should the memory be nulled?

        self.alloc_tabel.delete(ptr)
    }
}

#[derive(Eq,Clone)]
pub struct Allocation {ptr:u64, size:u64}

pub struct AllocationTable
{
    /// ## List of allocations
    /// 1.  pointer to first elemet
    ///     this gets passed as the return value of malloc
    ///     and identifies the chunk of memory 
    ///     cointains the size of the allocation
    /// 2.  cointains the size of the allocation
    elem: Vec<Allocation>,
}

impl AllocationTable
{
    pub fn new() -> Self
    {
        Self { elem: vec![] }
    }

    pub fn len(&self) -> usize
    {
        self.elem.len()
    }

    pub fn peek(&self, index: usize) -> Allocation
    {
        self.elem[index].clone()
    }

    pub fn insert(&mut self,a: Allocation)
    {
        self.elem.push(a);
        self.elem.sort();
    }

    pub fn delete(&mut self, ptr: u64) 
    {
        for ii in 0..self.elem.len()
        {
            if self.elem[ii].ptr == ptr
            {
                self.elem.remove(ii);
                break
            }
        }
    }
}

impl Ord for Allocation
{
    fn cmp(&self, other:&Self) -> Ordering
    {
        self.ptr.cmp(&other.ptr)
    }
}

impl PartialOrd for Allocation
{
    fn partial_cmp(&self, other:&Self) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}

impl PartialEq for Allocation
{
    fn eq(&self, other:&Self) -> bool
    {
        self.ptr == other.ptr
    }
}

#[cfg(test)]
mod tests
{
    //use super::*;

    // #[test]
    // fn rom_read_at_0_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.rom = vec![1,2,3];

    //     assert_eq!(mm.read_from_address(0),None)
    // }

    // #[test]
    // fn rom_read_at_max_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.rom = vec![1,2,3];

    //     assert_eq!(mm.read_from_address(2),Some(3))
    // }

    // #[test]
    // fn rom_read_on_emty_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.rom = vec![];

    //     assert_eq!(mm.read_from_address(2),None)
    // }

    // #[test]
    // fn code_read_at_0_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.rom = vec![1,2,3];
    //     mm.code = vec![4,5,6];

    //     assert_eq!(mm.read_from_address(3),Some(4))
    // }

    // #[test]
    // fn code_read_at_max_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.rom = vec![1,2,3];
    //     mm.code = vec![4,5,6];

    //     assert_eq!(mm.read_from_address(5),Some(6))
    // }

    // #[test]
    // fn code_read_on_emty_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.rom = vec![1,2,3];
    //     mm.code = vec![];

    //     assert_eq!(mm.read_from_address(3),None)
    // }

    // #[test]
    // fn code_read_with_no_rom_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.rom = vec![];
    //     mm.code = vec![1,2,3,4];

    //     assert_eq!(mm.read_from_address(3),Some(4))
    // }

    // #[test]
    // fn heap_read_basic_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.rom = vec![1,2];
    //     mm.code = vec![1,2,3,4];

    //     let he = HeapElem{ptr:6,data: vec![1,2,3]};
    //     mm.heap.push(he);

    //     assert_eq!(mm.read_from_address(7),Some(2))
    // }

    // #[test]
    // fn stack_read_basic_test()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.stack = vec![1,2,3,4,5];

    //     assert_eq!(mm.read_from_address(u64::MAX-4),Some(5))
    // }

    // #[test]
    // fn heap_remove_middle_test() -> Result<(),()>
    // {
    //     let mut mm = MemModel::new(10);

    //     let he1 = HeapElem{ptr:6,data: vec![1,2,3]};
    //     let he2 = HeapElem{ptr:345345,data: vec![1,3]};
    //     let he3 = HeapElem{ptr:2342342,data: vec![234,33]};

    //     mm.heap.push(he1);
    //     mm.heap.push(he2);
    //     mm.heap.push(he3);

    //     mm.heap_free(345345);

    //     if mm.heap[0].ptr == 6 && mm.heap[1].ptr == 2342342
    //     {
    //         Ok(())
    //     }
    //     else
    //     {
    //         Err(())
    //     }
    // }

    // #[test]
    // fn heap_remove_first_test() -> Result<(),()>
    // {
    //     let mut mm = MemModel::new(10);

    //     let he1 = HeapElem{ptr:6,data: vec![1,2,3]};
    //     let he2 = HeapElem{ptr:345345,data: vec![1,3]};
    //     let he3 = HeapElem{ptr:2342342,data: vec![234,33]};

    //     mm.heap.push(he1);
    //     mm.heap.push(he2);
    //     mm.heap.push(he3);

    //     mm.heap_free(6);

    //     if mm.heap[0].ptr == 345345 && mm.heap[1].ptr == 2342342
    //     {
    //         Ok(())
    //     }
    //     else
    //     {
    //         Err(())
    //     }
    // }

    // #[test]
    // fn heap_remove_last_test() -> Result<(),()>
    // {
    //     let mut mm = MemModel::new(10);

    //     let he1 = HeapElem{ptr:6,data: vec![1,2,3]};
    //     let he2 = HeapElem{ptr:345345,data: vec![1,3]};
    //     let he3 = HeapElem{ptr:2342342,data: vec![234,33]};

    //     mm.heap.push(he1);
    //     mm.heap.push(he2);
    //     mm.heap.push(he3);

    //     mm.heap_free(2342342);

    //     if mm.heap[0].ptr == 6 && mm.heap[1].ptr == 345345
    //     {
    //         Ok(())
    //     }
    //     else
    //     {
    //         Err(())
    //     }
    // }

    // #[test]
    // fn heap_malloc_on_empty_heap()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.code.push(0);

    //     assert_eq!(mm.heap_malloc(9),Some(1)); 
    // }

    // #[test]
    // fn heap_malloc_not_enough_mem()
    // {
    //     let mut mm = MemModel::new(10);

    //     mm.code.push(0);

    //     assert_eq!(mm.heap_malloc(10),None); 
    // }

    // #[test]
    // fn heap_malloc_enough_mem_on_not_empty_at_end_of_heap()
    // {
    //     let mut mm = MemModel::new(20);

    //     mm.code.push(0);
    //     //mm.stack.push(0);

    //     mm.heap.push(HeapElem{ptr:6,data: vec![1,2,3,4]});

    //     assert_eq!(mm.heap_malloc(10),Some(10)); 
    // }

    // #[test]
    // fn heap_malloc_between_elements()
    // {
    //     let mut mm = MemModel::new(20);
    //     mm.code.push(0);

    //     mm.heap.push(HeapElem{ptr:4,data: vec![1,2,3,4,5,6]});
    //     mm.heap.push(HeapElem{ptr:15,data: vec![1,2,3,4]});

    //     assert_eq!(mm.heap_malloc(5),Some(10));
    // }
}
