//! This module implements the `Application` trait with the corresponding function definitions found in the C code base of `browser-window-c`.
//! All functions are basically wrapping the FFI provided by crate `browser-window-c`.

use super::ApplicationExt;

use crate::prelude::*;

use std::os::raw::{c_char, c_int, c_void};



#[derive(Clone,Copy)]
pub struct ApplicationImpl {
	pub(in crate) inner: *mut cbw_Application
}

impl ApplicationExt for ApplicationImpl {

	fn dispatch( &self, work: unsafe fn(ApplicationImpl, *mut ()), _data: *mut () ) -> bool {
		let data = Box::new( DispatchData {
			func: work,
			data: _data
		} );

		let data_ptr = Box::into_raw( data );

		unsafe { cbw_Application_dispatch( self.inner, Some( invocation_handler ), data_ptr as _ ) != 0 }
	}
	
	fn exit( &self, exit_code: i32 ) {
		unsafe { cbw_Application_exit( self.inner, exit_code as _ ) }
	}
	
	fn exit_threadsafe( self: &Self, exit_code: i32 ) {
		unsafe { cbw_Application_exitAsync( self.inner, exit_code ) }
	}
	
	fn finish( &self ) {
		unsafe { cbw_Application_finish( self.inner ) }
	}

	fn initialize( argc: c_int, argv: *mut *mut c_char ) -> Self {
		let c_handle = unsafe { cbw_Application_initialize( argc, argv ) };

		Self {
			inner: c_handle
		}
	}

	fn run( &self, on_ready: unsafe fn( ApplicationImpl, *mut () ), _data: *mut () ) -> i32 {
		let data = Box::new( DispatchData {
			func: on_ready,
			data: _data
		} );

		let data_ptr = Box::into_raw( data );

		// The dispatch handler does exactly the same thing 
		unsafe { cbw_Application_run( self.inner, Some( invocation_handler ), data_ptr as _ ) }
	}
}



struct DispatchData {
	func: unsafe fn( ApplicationImpl, *mut () ),
	data: *mut ()
}

unsafe extern "C" fn invocation_handler( _handle: *mut cbw_Application, _data: *mut c_void ) {

	let data_ptr = _data as *mut DispatchData;
	let data = Box::from_raw( data_ptr );
	let handle = ApplicationImpl { inner: _handle };

	(data.func)( handle, data.data );
}