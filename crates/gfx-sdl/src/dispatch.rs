pub type DispatchableUserdata = *mut ();
pub type DispatchableFunc = extern "C" fn(userdata: DispatchableUserdata);
pub type DispatcherFunc = extern "C" fn(func: DispatchableFunc, userdata: DispatchableUserdata);

#[inline]
pub fn dispatch<Ret: Sized, Func: FnOnce() -> Ret + Copy>(
    dispatcher: Option<DispatcherFunc>,
    func: Func,
) -> Ret {
    if let Some(dispatcher) = dispatcher {
        let mut ret = std::mem::MaybeUninit::<Ret>::uninit();

        struct Userdata<'a, Ret: Sized, Func: FnOnce() -> Ret> {
            pub func: Func,
            pub ret: &'a mut std::mem::MaybeUninit<Ret>,
        }

        let mut userdata = Userdata {
            func,
            ret: &mut ret,
        };

        extern "C" fn dispatchable<Ret: Sized, Func: FnOnce() -> Ret + Copy>(
            userdata: DispatchableUserdata,
        ) {
            let userdata = userdata as *mut Userdata<'_, Ret, Func>;
            let userdata = unsafe { &mut *userdata };
            userdata.ret.write((userdata.func)());
        }

        dispatcher(
            dispatchable::<Ret, Func>,
            (&mut userdata as *mut Userdata<'_, Ret, Func>) as DispatchableUserdata,
        );
        unsafe { ret.assume_init() }
    } else {
        func()
    }
}
