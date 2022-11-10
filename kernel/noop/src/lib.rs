/* This simplest kernel returns directly after being called,
   without doing anything particular */
#[no_mangle]
pub extern "C" fn kernel_next() {}