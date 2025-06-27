use crate::vmcontext::{VMFunctionContext, VMTrampoline};
use crate::{Trap, VMContext, VMFunctionBody};
use std::any::Any;
use std::error::Error;
use std::mem;

/// Dummy trap handler type for baremetal mode
pub type TrapHandlerFn<'a> = ();
/// Dummy config type for baremetal mode
pub struct VMConfig {
    /// This is put here to make compile happy, we don't really use this
    /// value.
    pub wasm_stack_size: Option<usize>,
}

/// Baremetal does not support setting stack size, the function is kept
/// here to preserve APIs
pub fn set_stack_size(_size: usize) {
    panic!("Setting stack size is not supported in baremetal feature!");
}

/// In baremetal mode, init_traps does nothing
pub fn init_traps() {}

/// In baremetal mode, on_host_stack does not addition action
pub fn on_host_stack<F: FnOnce() -> T, T>(f: F) -> T {
    f()
}

/// In baremetal mode, catch_traps simply ignores trap handler and config
pub unsafe fn catch_traps<F, R: 'static>(
    _trap_handler: Option<*const TrapHandlerFn<'static>>,
    _config: &VMConfig,
    closure: F,
) -> Result<R, Trap>
where
    F: FnOnce() -> R + 'static,
{
    Ok(closure())
}

/// In baremetal mode, resuming panic also panics for now.
pub unsafe fn resume_panic(payload: Box<dyn Any + Send>) -> ! {
    panic!("Resuming panic: {:?}", payload);
}

/// In baremetal mode, raising user trap simply panics
pub unsafe fn raise_user_trap(data: Box<dyn Error + Send + Sync>) -> ! {
    panic!("User trap: {}", data);
}

/// In baremetal mode, raising lib trap simply panics
pub unsafe fn raise_lib_trap(trap: Trap) -> ! {
    panic!("Lib trap: {:?}", trap);
}

/// When the inner functions have been mocked, wasmer_call_trampoline
/// in baremetal mode can be implemented exactly as it is in os mode.
pub unsafe fn wasmer_call_trampoline(
    trap_handler: Option<*const TrapHandlerFn<'static>>,
    config: &VMConfig,
    vmctx: VMFunctionContext,
    trampoline: VMTrampoline,
    callee: *const VMFunctionBody,
    values_vec: *mut u8,
) -> Result<(), Trap> {
    catch_traps(trap_handler, config, move || {
        mem::transmute::<
            unsafe extern "C" fn(
                *mut VMContext,
                *const VMFunctionBody,
                *mut wasmer_types::RawValue,
            ),
            extern "C" fn(VMFunctionContext, *const VMFunctionBody, *mut u8),
        >(trampoline)(vmctx, callee, values_vec);
    })
}
