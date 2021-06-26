// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A Postgres Allocator

use std::ffi::{CString, c_void};
use std::marker::{PhantomData, PhantomPinned};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use crate::pg_sys;

/// Provides memory allocation which is wholly managed by Postgres' `MemoryContext`s
pub struct PgMemoryContext {
    memcxt: pg_sys::MemoryContext,
    savedcxt: Option<pg_sys::MemoryContext>,
}

impl PgMemoryContext {

    //
    // allocation actions to be performed within this `PgMemoryContext`
    //

    /// Allocate memory in this `MemoryContext` and return a pointer it
    pub fn alloc(&self, size: usize) -> *mut std::os::raw::c_void {
        unsafe {
            pg_sys::MemoryContextAlloc(self.memcxt, size)
        }
    }

    /// Allocate memory in this `MemoryContext` and return a pointer to it
    ///
    /// ensures that all allocated bytes are zero'd
    pub fn alloc0(&self, size: usize) -> *mut std::os::raw::c_void {
        unsafe {
            pg_sys::MemoryContextAllocZero(self.memcxt, size)
        }
    }

    /// Free Postgres-allocated memory, regardless of the `MemoryContext`
    /// in which it was allocated
    pub fn pfree(ptr: *mut std::os::raw::c_void) {
        unsafe {
            pg_sys::pfree(ptr);
        }
    }


    //
    // whole-context management functions
    //

    /// Free's all memory allocated in this context and all child contexts, but keeps it usable
    pub fn reset(&self) {
        unsafe {
            pg_sys::MemoryContextReset(self.memcxt)
        }
    }

    /// Free's all memory allocated in this context only, leaving child contexts untouched
    pub fn reset_only(&self) {
        unsafe {
            pg_sys::MemoryContextResetOnly(self.memcxt)
        }
    }

    /// Deletes this memory context and all child contexts by freeing all allocated memory.
    /// Afterwards, it is no longer usable
    pub fn delete(self) {
        unsafe {
            pg_sys::MemoryContextDelete(self.memcxt);
        }
    }


    //
    // functions to make the wrapped `MemoryContext` the active Postgres `CurrentMemoryContext`
    //

    /// Switch to this `MemoryContext`.
    ///
    /// Prior to switching Postgres' `CurrentMemoryContext` will be remembered and automatically
    /// restored when this instance is dropped or otherwise goes out of scope.
    ///
    /// Note that this does not mean that the memory allocated is freed, only that we restore
    /// the context stack
    pub fn switch_to(&mut self) -> &mut Self {
        unsafe {
            self.savedcxt = Some(pg_sys::CurrentMemoryContext);
            pg_sys::CurrentMemoryContext = self.memcxt;
        }
        self
    }

    /// Execute code entirely within this `MemoryContext`
    pub fn exec_in_context<R, F: FnOnce() -> R>(mut self, f: F) -> R {
        self.switch_to();
        f()
    }


    //
    // functions for retrieving a specific Postgres `MemoryContext`
    //

    /// Create a named and sized `MemoryContext` that is a child of this `MemoryContext`
    pub fn create(&self, name: &'static str, min_context_size: usize, initial_block_size: usize, max_block_size: usize) -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::AllocSetContextCreateExtended(
                    self.memcxt,
                    CString::new(name).expect("invalud MemoryContextName").as_ptr(),
                    min_context_size,
                    initial_block_size,
                    max_block_size),
                savedcxt: None,
            }
        }
    }

    /// Create a named `MemoryContext` of default Postgres sizes, that is a child of this `MemoryContext`
    pub fn create_with_defaults(&self, name: &'static str) -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::AllocSetContextCreateExtended(
                    self.memcxt,
                    CString::new(name).expect("invalud MemoryContextName").as_ptr(),
                    pg_sys::ALLOCSET_DEFAULT_MINSIZE as usize,
                    pg_sys::ALLOCSET_DEFAULT_INITSIZE as usize,
                    pg_sys::ALLOCSET_DEFAULT_MAXSIZE as usize),
                savedcxt: None,
            }
        }
    }

    /// Create a `PgMemoryContext` from a raw Postgres `MemoryContext`
    pub fn from_raw(memcxt: pg_sys::MemoryContext) -> Self {
        PgMemoryContext {
            memcxt: memcxt,
            savedcxt: None,
        }
    }

    /// Retrieves a reference to the `CurrentMemoryContext`
    ///
    /// At all times there is a "current" context denoted by the
    /// CurrentMemoryContext global variable.  palloc() implicitly allocates space
    /// in that context.  The MemoryContextSwitchTo() operation selects a new current
    /// context (and returns the previous context, so that the caller can restore the
    /// previous context before exiting).
    pub fn current() -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::CurrentMemoryContext,
                savedcxt: None,
            }
        }
    }

    /// Retrieves a reference to the `TopMemoryContext`
    ///
    /// TopMemoryContext --- this is the actual top level of the context tree;
    /// every other context is a direct or indirect child of this one.  Allocating
    /// here is essentially the same as "malloc", because this context will never
    /// be reset or deleted.  This is for stuff that should live forever, or for
    /// stuff that the controlling module will take care of deleting at the
    /// appropriate time.  An example is fd.c's tables of open files.  Avoid
    /// allocating stuff here unless really necessary, and especially avoid
    /// running with CurrentMemoryContext pointing here.
    pub fn top() -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::TopMemoryContext,
                savedcxt: None,
            }
        }
    }

    /// Retrieves a reference to the `ErrorContext`
    ///
    /// ErrorContext --- this permanent context is switched into for error
    /// recovery processing, and then reset on completion of recovery.  We arrange
    /// to have a few KB of memory available in it at all times.  In this way, we
    /// can ensure that some memory is available for error recovery even if the
    /// backend has run out of memory otherwise.  This allows out-of-memory to be
    /// treated as a normal ERROR condition, not a FATAL error.
    pub fn error() -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::ErrorContext,
                savedcxt: None,
            }
        }
    }

    /// Retrieves a reference to the `PostmasterContext`
    ///
    /// PostmasterContext --- this is the postmaster's normal working context.
    /// After a backend is spawned, it can delete PostmasterContext to free its
    /// copy of memory the postmaster was using that it doesn't need.
    /// Note that in non-EXEC_BACKEND builds, the postmaster's copy of pg_hba.conf
    /// and pg_ident.conf data is used directly during authentication in backend
    /// processes; so backends can't delete PostmasterContext until that's done.
    /// (The postmaster has only TopMemoryContext, PostmasterContext, and
    /// ErrorContext --- the remaining top-level contexts are set up in each
    /// backend during startup.)
    pub fn postmaster() -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::PostmasterContext,
                savedcxt: None,
            }
        }
    }

    /// Retrieves a reference to the `CacheMemoryContext`
    ///
    /// CacheMemoryContext --- permanent storage for relcache, catcache, and
    /// related modules.  This will never be reset or deleted, either, so it's
    /// not truly necessary to distinguish it from TopMemoryContext.  But it
    /// seems worthwhile to maintain the distinction for debugging purposes.
    /// (Note: CacheMemoryContext has child contexts with shorter lifespans.
    /// For example, a child context is the best place to keep the subsidiary
    /// storage associated with a relcache entry; that way we can free rule
    /// parsetrees and so forth easily, without having to depend on constructing
    /// a reliable version of freeObject().)
    pub fn cache() -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::CacheMemoryContext,
                savedcxt: None,
            }
        }
    }

    /// Retrieves a reference to the `MessageContext`
    ///
    /// MessageContext --- this context holds the current command message from the
    /// frontend, as well as any derived storage that need only live as long as
    /// the current message (for example, in simple-Query mode the parse and plan
    /// trees can live here).  This context will be reset, and any children
    /// deleted, at the top of each cycle of the outer loop of PostgresMain.  This
    /// is kept separate from per-transaction and per-portal contexts because a
    /// query string might need to live either a longer or shorter time than any
    /// single transaction or portal.
    pub fn message() -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::MessageContext,
                savedcxt: None,
            }
        }
    }

    /// Retrieves a reference to the `TopTransactionContext`
    ///
    /// TopTransactionContext --- this holds everything that lives until end of the
    /// top-level transaction.  This context will be reset, and all its children
    /// deleted, at conclusion of each top-level transaction cycle.  In most cases
    /// you don't want to allocate stuff directly here, but in CurTransactionContext;
    /// what does belong here is control information that exists explicitly to manage
    /// status across multiple subtransactions.  Note: this context is NOT cleared
    /// immediately upon error; its contents will survive until the transaction block
    /// is exited by COMMIT/ROLLBACK.
    pub fn top_transaction() -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::TopTransactionContext,
                savedcxt: None,
            }
        }
    }

    /// Retrieves a reference to the `CurrentTransactionContext`
    ///
    /// CurTransactionContext --- this holds data that has to survive until the end
    /// of the current transaction, and in particular will be needed at top-level
    /// transaction commit.  When we are in a top-level transaction this is the same
    /// as TopTransactionContext, but in subtransactions it points to a child context.
    /// It is important to understand that if a subtransaction aborts, its
    /// CurTransactionContext is thrown away after finishing the abort processing;
    /// but a committed subtransaction's CurTransactionContext is kept until top-level
    /// commit (unless of course one of the intermediate levels of subtransaction
    /// aborts).  This ensures that we do not keep data from a failed subtransaction
    /// longer than necessary.  Because of this behavior, you must be careful to clean
    /// up properly during subtransaction abort --- the subtransaction's state must be
    /// delinked from any pointers or lists kept in upper transactions, or you will
    /// have dangling pointers leading to a crash at top-level commit.  An example of
    /// data kept here is pending NOTIFY messages, which are sent at top-level commit,
    /// but only if the generating subtransaction did not abort.
    pub fn current_transaction() -> Self {
        unsafe {
            PgMemoryContext {
                memcxt: pg_sys::CurTransactionContext,
                savedcxt: None,
            }
        }
    }
}

impl Drop for PgMemoryContext {

    /// When a `PgMemoryContext` is dropped, which is just a lightweight Rust wrapper
    /// around a Postgres-managed `MemoryContext` pointer, we need to switch back
    /// the `MemoryContext` that was active before ``PgMemoryContext.switch_to()`` was called
    ///
    /// If `switch_to()` was never called, then there's nothing we need to do here
    fn drop(&mut self) {
        match self.savedcxt {
            Some(savedcxt) => unsafe { pg_sys::CurrentMemoryContext = savedcxt },
            None => ()
        }
    }
}

/// An allocattor which uses the palloc and pfree functions available from Postgres.
///
/// This is managed by Postgres and guarantees that all memory is freed after a transaction completes.
pub struct PgAllocator(NonNull<pg_sys::MemoryContextData>);

impl PgAllocator {
    /// Instantiate a PgAllocator from the raw pointer.
    unsafe fn from_raw(context: *mut pg_sys::MemoryContextData) -> Self {
        Self(NonNull::new_unchecked(context))
    }

    /// Establishes a PgAllocator from the current default context.
    pub fn current_context() -> Self {
        unsafe { Self::from_raw(pg_sys::CurrentMemoryContext) }
    }

    /// Sets this PgAllocator as the current memory context, and then resets it to the previous
    /// after executing the function.
    pub fn exec<R, F: FnOnce() -> R>(&self, f: F) -> R {
        let previous_context;
        unsafe {
            // save the previous context
            previous_context = pg_sys::CurrentMemoryContext;

            // set this context as the current
            pg_sys::CurrentMemoryContext = self.0.as_ref() as *const _ as *mut _;
        }

        // TODO: should we catch panics here to guarantee the context is reset?
        let result = f();

        // reset the previous context
        unsafe {
            pg_sys::CurrentMemoryContext = previous_context;
        }

        result
    }

    /// Same as exec, but additionally wraps in with pg_guard
    ///
    /// # Safety
    ///
    /// This has the same safety requirements as `guard_pg`
    pub unsafe fn exec_with_guard<R, F: FnOnce() -> R>(&self, f: F) -> R {
        use crate::guard_pg;

        self.exec(|| guard_pg(f))
    }

    unsafe fn dealloc<T: ?Sized>(&self, pg_data: *mut T) {
        // TODO: see mctx.c in Postgres' source this probably needs more validation
        let ptr = pg_data as *mut c_void;
        //  pg_sys::pfree(pg_data as *mut c_void)
        let methods = *self.0.as_ref().methods;
        crate::guard_pg(|| {
            methods.free_p.expect("free_p is none")(self.0.as_ref() as *const _ as *mut _, ptr);
        });
    }
}

/// Types that were allocated by Postgres
///
/// Any data allocated by Postgres or being returned to Postgres for management must be stored in this value.
pub struct PgAllocated<'mc, T: 'mc + RawPtr> {
    inner: Option<ManuallyDrop<T>>,
    allocator: &'mc PgAllocator,
    _disable_send_sync: PhantomData<NonNull<&'mc T>>,
    _not_unpin: PhantomPinned,
}

impl<'mc, T: RawPtr> PgAllocated<'mc, T>
    where
        T: 'mc + RawPtr,
{
    /// Creates a new Allocated type from Postgres.
    ///
    /// This does not allocate, it associates the lifetime of the Allocator to this type.
    ///   it protects the wrapped type from being dropped by Rust, and uses the
    ///   associated Postgres Allocator for freeing the backing memory.
    ///
    /// # Safety
    ///
    /// The memory referenced by `ptr` must have been allocated withinn the associated `memory_context`.
    pub unsafe fn from_raw(
        memory_context: &'mc PgAllocator,
        ptr: *mut <T as RawPtr>::Target,
    ) -> Self {
        PgAllocated {
            inner: Some(ManuallyDrop::new(T::from_raw(ptr))),
            allocator: memory_context,
            _disable_send_sync: PhantomData,
            _not_unpin: PhantomPinned,
        }
    }

    /// This consumes the inner pointer
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn take_ptr(&mut self) -> *mut <T as RawPtr>::Target {
        let inner = self
            .inner
            .take()
            .expect("invalid None while PgAllocated is live");
        ManuallyDrop::into_inner(inner).into_raw()
    }

    /// Returns a pointer to the inner type
    pub fn as_ptr(&self) -> *const <T as RawPtr>::Target {
        self.inner
            .as_ref()
            .expect("invalid None while PgAllocated is live")
            .as_ptr()
    }
}

impl<'mc, T: 'mc + RawPtr> Deref for PgAllocated<'mc, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
            .as_ref()
            .expect("invalid None while PgAllocated is live")
            .deref()
    }
}

impl<'mc, T: 'mc + RawPtr> DerefMut for PgAllocated<'mc, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO: instead of requiring Option here, swap the pointer with 0, and allow free on 0, which is safe.
        self.inner
            .as_mut()
            .expect("invalid None while PgAllocated is live")
            .deref_mut()
    }
}

impl<'mc, T: RawPtr> Drop for PgAllocated<'mc, T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            unsafe {
                // TODO: do we need to run the drop on the inner type?
                // let ptr: *mut T = mem::transmute(inner.deref_mut().deref_mut());
                let ptr: *mut _ = ManuallyDrop::into_inner(inner).into_raw();
                self.allocator.dealloc(ptr);
            }
        }
    }
}

/// Types which implement this can be converted from pointers to their Rust type and vice versa.
pub trait RawPtr {
    /// Type to which the pointer is associated.
    type Target;

    /// Instantiate the type from the pointer
    ///
    /// # Safety
    ///
    /// Implementors should validate that all conversions into Rust wrapper type are within MemoryContexts
    unsafe fn from_raw(ptr: *mut Self::Target) -> Self;

    /// Consume this and return the pointer.
    ///
    /// # Safety
    ///
    /// After calling `into_raw` there should be no other pointers to the data behind the pointer.
    unsafe fn into_raw(self) -> *mut Self::Target;

    /// Returns a pointer to this
    fn as_ptr(&self) -> *const Self::Target;
}

impl RawPtr for std::ffi::CString {
    type Target = std::os::raw::c_char;

    unsafe fn from_raw(ptr: *mut Self::Target) -> Self {
        std::ffi::CString::from_raw(ptr)
    }

    unsafe fn into_raw(self) -> *mut Self::Target {
        std::ffi::CString::into_raw(self)
    }

    fn as_ptr(&self) -> *const Self::Target {
        self.as_c_str().as_ptr()
    }
}

impl RawPtr for NonNull<pg_sys::text> {
    type Target = pg_sys::text;

    unsafe fn from_raw(ptr: *mut Self::Target) -> Self {
        NonNull::new_unchecked(ptr)
    }

    unsafe fn into_raw(self) -> *mut Self::Target {
        NonNull::as_ptr(self)
    }

    fn as_ptr(&self) -> *const Self::Target {
        unsafe { self.as_ref() }
    }
}
