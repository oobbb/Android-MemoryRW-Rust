use std::{thread, time};

mod MemoryTools;























fn main() {
    let pid = MemoryTools::get_pid("com.proximabeta.nikke");
    if pid == -1 {
        println!("获取pid失败.");
    } else {
        println!("pid为 {}.", pid);
    }
    
    //获取模块基址
    let module_base = MemoryTools::get_module_base(pid, "libil2cpp.so", 1);
    println!("基址: {:X}", module_base);


    let addr: i64 = 0x6C703DA08C;
    
   // let addr: i64 = module_base as i64;
    
    //32位指针读取
    let value_32 = MemoryTools::read_pointer_32(pid, addr);
    
    //64位指针读取
    let value_64 = MemoryTools::read_pointer_64(pid, addr);
    
    
    //int
    let int_test = MemoryTools::read_value::<i32>(pid, addr);
    
    
    //float
    let float_test = MemoryTools::read_value::<f32>(pid, addr);
    
    


    
    thread::sleep(time::Duration::from_secs(5));
    MemoryTools::Edit::<i32>(pid,addr,666666);
    println!("修改成功");

}