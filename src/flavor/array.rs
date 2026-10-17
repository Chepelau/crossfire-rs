use super::{FlavorBounded, FlavorImpl, FlavorMC, FlavorMP, FlavorSelect, Queue, Token};
use crate::crossbeam::array_queue::ArrayQueue;
use crossfire_utils::backoff;
use std::mem::MaybeUninit;

/// Which Equals to crossbeam_queue::ArrayQueue
pub struct Array<T>(ArrayQueue<T, true, true>);

impl<T> Array<T> {
    pub fn new(mut bound: usize) -> Self {
        assert!(bound <= u32::MAX as usize);
        if bound == 0 {
            bound = 1;
        }
        Self(ArrayQueue::<T, true, true>::new(bound))
    }
}

impl<T> Queue for Array<T> {
    type Item = T;

    #[inline(always)]
    fn pop(&self) -> Option<T> {
        self.0.pop(true)
    }

    #[inline(always)]
    fn push(&self, item: T) -> Result<(), T> {
        let _item = MaybeUninit::new(item);
        if unsafe { self.0.push_with_ptr(_item.as_ptr()) } {
            Ok(())
        } else {
            Err(unsafe { _item.assume_init_read() })
        }
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        self.0.is_full()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    fn capacity(&self) -> Option<usize> {
        Some(self.0.capacity())
    }
}

impl<T> FlavorImpl for Array<T> {
    const IS_BOUNDED: bool = true;

    #[inline(always)]
    fn try_send(&self, item: &MaybeUninit<T>) -> bool {
        unsafe { self.0.push_with_ptr(item.as_ptr()) }
    }

    #[inline(always)]
    fn try_send_oneshot(&self, item: *const T) -> Option<bool> {
        unsafe { self.0.try_push_oneshot(item) }
    }

    #[inline]
    fn try_recv(&self) -> Option<T> {
        self.0.pop(false)
    }

    #[inline]
    fn try_recv_final(&self) -> Option<T> {
        self.0.pop(true)
    }

    #[inline]
    fn backoff_limit() -> u16 {
        #[cfg(target_arch = "x86_64")]
        {
            backoff::DEFAULT_LIMIT
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            backoff::MAX_LIMIT
        }
    }

    #[inline]
    fn adjust_tx_backoff(&self, back: &mut backoff::Backoff) {
        if self.0.capacity() > 20 {
            back.set_step(2);
            back.yield_now();
        } else {
            #[cfg(not(target_arch = "x86_64"))]
            {
                back.config.limit = backoff::MAX_LIMIT;
            }
        }
    }
}

impl<T> FlavorSelect for Array<T> {
    #[inline]
    fn try_select(&self, final_check: bool) -> Option<Token> {
        self.0.start_read(final_check)
    }

    #[inline(always)]
    fn read_with_token(&self, token: Token) -> T {
        self.0.read(token)
    }
}

impl<T> FlavorBounded for Array<T> {
    #[inline(always)]
    fn new_with_bound(size: usize) -> Self {
        Self::new(size)
    }
}

impl<T> FlavorMP for Array<T> {}
impl<T> FlavorMC for Array<T> {}
