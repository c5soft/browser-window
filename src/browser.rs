use boxfnonce::SendBoxFnOnce;
use browser_window_ffi::*;
use futures_channel::oneshot;
use std::{
	error::Error,
	ffi::CStr,
	fmt,
	marker::PhantomData,
	ops::Deref,
	os::raw::*,
	rc::Rc
};

use crate::application::*;
use crate::common::*;

pub mod builder;

pub use builder::BrowserBuilder;



type BrowserJsCallbackData<'a> = Box<dyn FnOnce(Browser, Result<String, JsEvaluationError>) + 'a>;
type BrowserJsThreadedCallbackData<'a> = SendBoxFnOnce<'a,(BrowserHandle, Result<String, JsEvaluationError>),()>;

/// The future that dispatches a closure on the GUI thread.
pub type BrowserDispatchFuture<'a,R> = DispatchFuture<'a, BrowserHandle, R>;



/// A thread-unsafe handle to a browser window
// If the user closes the window, this handle remains valid.
// Also, if you lose this handle, window destruction and cleanup is only done when the user actually closes it.
// So you don't have to worry about lifetimes and/or propper destruction of the window either.
pub struct Browser {
	pub(in super) handle: BrowserHandle,
	_not_send: PhantomData<Rc<()>>
}

/// A thread-safe handle to a browser window.
/// It allows you to dispatch code to the GUI thread.
// It provides the same functionality as `Browser`.
// However, each function is async: it runs on the GUI thread, and returns when it is done.
pub struct BrowserThreaded {
	pub(in super) handle: BrowserHandle
}
unsafe impl Sync for BrowserThreaded {}

#[derive(Clone)]
pub struct BrowserHandle {
	pub(in super) ffi_handle: *mut bw_BrowserWindow
}
unsafe impl Send for BrowserHandle {}

/// An error that may occur when evaluating or executing JavaScript code.
#[derive(Debug)]
pub struct JsEvaluationError {
	message: String
	// TODO: Add line and column number files, and perhaps even more info about the JS error
}



impl Browser {

	/// Returns the application handle associated with this browser window.
	pub fn app( &self ) -> Application {
		Application::from_ffi_handle( unsafe { bw_BrowserWindow_getApp( self.handle.ffi_handle ) } )
	}

	/// Closes the browser.
	// The browser will be freed from memory when the last handle to it gets dropped.
	pub fn close( self ) {
		unsafe { bw_BrowserWindow_close( self.handle.ffi_handle ); }
	}

	pub async fn eval_js( &self, js: &str ) -> Result<String, JsEvaluationError> {
		let (tx, rx) = oneshot::channel::<Result<String, JsEvaluationError>>();

		self._eval_js( js, |_, result| {
			if let Err(_) = tx.send( result ) {
				panic!("Unable to send JavaScript result back")
			}
		} );

		rx.await.unwrap()
	}


	/// Executes the given javascript code, and returns the output via a callback.
	/// If you don't need the result, see "exec_js".
	///
	/// # Arguments:
	/// * `js` - The javascript code to execute.
	/// * `on_complete` - The 'callback'. This closure will be invoked, with the result provided as the first argument.
	///                   The result contains the output of the javascript code when it succeeded.
	///                   Otherwise the error explains the javascript exception.
	fn _eval_js<'a,H>( &self, js: &str, on_complete: H ) where
		H: FnOnce( Browser, Result<String, JsEvaluationError> ) + 'a
	{
		let data_ptr: *mut H = Box::into_raw(
			Box::new( on_complete )
		);

		unsafe { bw_BrowserWindow_evalJs(
			self.handle.ffi_handle,
			js.into(),
			ffi_eval_js_callback::<H>,
			data_ptr as _
		) };
	}

	/// Executes the given javascript code without waiting on it to finish.
	///
	/// # Arguments:
	/// * `js` - The javascript code
	pub fn exec_js( &self, js: &str ) {
		self._eval_js( js, |_,_|{} );
	}

	fn from_ffi_handle( ptr: *mut bw_BrowserWindow ) -> Self {
		Self {
			handle: BrowserHandle::new( ptr ),
			_not_send: PhantomData
		}
	}

	/// Causes the browser to navigate to the given url.
	///
	/// # Arguments
	/// * `url` - The url to navigate to
	pub fn navigate( &self, url: &str ) -> Result<(), Box<dyn Error + Send>> {
		let err = unsafe { bw_BrowserWindow_navigate( self.handle.ffi_handle, url.into() ) };

		if err.code == 0 {
			return Ok(());
		}

		Err( Box::new( err ) )
	}
}

impl Deref for Browser {
	type Target = BrowserHandle;

	fn deref( &self ) -> &BrowserHandle {
		&self.handle
	}
}

impl Drop for Browser {
	fn drop( &mut self ) {
		unsafe { bw_BrowserWindow_drop( self.handle.ffi_handle ) }
	}
}

impl From<BrowserHandle> for Browser {

	fn from( handle: BrowserHandle ) -> Self {
		Self {
			handle: handle,
			_not_send: PhantomData
		}
	}
}

impl HasAppHandle for Browser {

	fn app_handle( &self ) -> ApplicationHandle {
		self.handle.app_handle()
	}
}



impl BrowserThreaded {

	pub fn app( &self ) -> ApplicationThreaded {
		ApplicationThreaded::from_ffi_handle( unsafe { bw_BrowserWindow_getApp( self.handle.ffi_handle ) } )
	}

	/// Closes the browser.
	pub async fn close( self ) {
		self.dispatch(|bw| {
			bw.close()
		}).await;
	}

	/// Executes the given closure within the GUI thread, and return the value that the closure returned.
	/// Keep in mind that in multi-threaded environments, it is generally a good idea to Box return type,
	///  or use something else to put the value on the heap when dealing with large types.
	///
	/// # Arguments
	/// * `func` - The closure to run on the GUI thread.
	pub fn dispatch<'a,F,R>( &self, func: F ) -> BrowserDispatchFuture<'a,R> where
		F: FnOnce( Browser ) -> R + Send + 'a,
		R: Send
	{
		BrowserDispatchFuture::new( self.handle.clone(), |handle| {
			func( handle.into() )
		} )
	}

	/// Executes the given javascript code, and returns the resulting output as a string when done.
	///
	/// # Arguments:
	/// * `js` - Javascript code
	pub async fn eval_js( &self, js: &str ) -> Result<String, JsEvaluationError> {
		let (tx, rx) = oneshot::channel::<Result<String, JsEvaluationError>>();

		self._eval_js( js, |_, result| {
			if let Err(_) = tx.send( result ) {
				panic!("Unable to send JavaScript result back")
			}
		} );

		rx.await.unwrap()
	}

	/// Causes the browser to navigate to the given url.
	///
	/// # Arguments
	/// * `url` - The url to navigate to
	pub async fn navigate( &self, url: &str ) -> Result<(), Box<dyn Error + Send>> {
		self.dispatch(|bw| {
			bw.navigate( url )
		}).await
	}

	fn _eval_js<'a,H>( &self, js: &str, on_complete: H ) where
		H: FnOnce( BrowserThreaded, Result<String, JsEvaluationError> ) + Send + 'a
	{
		let data_ptr: *mut H = Box::into_raw(
			Box::new( on_complete )
		);

		unsafe { bw_BrowserWindow_evalJsThreaded(
			self.handle.ffi_handle,
			js.into(),
			ffi_eval_js_threaded_callback::<H>,
			data_ptr as _
		) };
	}
}

impl Deref for BrowserThreaded {
	type Target = BrowserHandle;

	fn deref( &self ) -> &BrowserHandle {
		&self.handle
	}
}

impl Drop for BrowserThreaded {
	fn drop( &mut self ) {
		unsafe { bw_Application_dispatch( self.app().handle.ffi_handle, ffi_free_browser_window, self.handle.ffi_handle as _ ); }
	}
}

impl From<BrowserHandle> for BrowserThreaded {

	fn from( handle: BrowserHandle ) -> Self {
		Self {
			handle: handle
		}
	}
}

impl HasAppHandle for BrowserThreaded {

	fn app_handle( &self ) -> ApplicationHandle {
		self.handle.app_handle()
	}
}



impl BrowserHandle {
	fn new( ffi_handle: *mut bw_BrowserWindow ) -> Self {
		Self {
			ffi_handle: ffi_handle
		}
	}
}

impl HasAppHandle for BrowserHandle {

	fn app_handle( &self ) -> ApplicationHandle {
		ApplicationHandle::new(
			unsafe { bw_BrowserWindow_getApp( self.ffi_handle ) }
		)
	}
}



impl JsEvaluationError {
	pub(in super) unsafe fn new( err: *const bw_Err ) -> Self {

		let msg_ptr = ((*err).alloc_message)( (*err).code, (*err).data );
		let cstr = CStr::from_ptr( msg_ptr );
		let message: String = cstr.to_string_lossy().into();

		Self {
			message: message
		}
	}
}

impl Error for JsEvaluationError {
	fn source(&self) -> Option<&(dyn Error + 'static)> { None }
}

impl fmt::Display for JsEvaluationError {

	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

		write!(f, "{}", self.message.as_str())
	}
}



/// Callback for dropping a browser window.
/// This gets dispatch to the GUI thread when a `BrowserThreaded` handle gets dropped.
unsafe extern "C" fn ffi_free_browser_window( _app: *mut bw_Application, data: *mut c_void ) {
	unsafe { bw_BrowserWindow_drop( data as *mut bw_BrowserWindow ); }
}

unsafe extern "C" fn ffi_eval_js_callback<H>( bw: *mut bw_BrowserWindow, cb_data: *mut c_void, _result: *const c_char, error: *const bw_Err ) where
	H: FnOnce(Browser, Result<String, JsEvaluationError>)
{
	let data_ptr = cb_data as *mut H;
	let data = unsafe { Box::from_raw( data_ptr ) };

	let (handle, result) = ffi_eval_js_callback_result( bw, _result, error );

	(*data)( handle.into(), result );
}

unsafe fn ffi_eval_js_callback_result(
	bw: *mut bw_BrowserWindow,
	result: *const c_char,
	error: *const bw_Err
) -> ( BrowserHandle, Result<String, JsEvaluationError> ) {


	// Construct a result value depending on whether the result or error parameters are set
	let result_val: Result<String, JsEvaluationError> = if error.is_null() {
		let result_str = CStr::from_ptr( result ).to_string_lossy().to_owned().to_string();
		Ok( result_str )
	}
	else {
		Err( JsEvaluationError::new( error ) )
	};

	let handle = BrowserHandle::new( bw );

	// return
	( handle, result_val )
}

/// Callback for catching JavaScript results.
///
/// # Warning
/// This may get invoked from another thread than the GUI thread, depending on the implementation of the browser engine.
unsafe extern "C" fn ffi_eval_js_threaded_callback<H>( bw: *mut bw_BrowserWindow, cb_data: *mut c_void, _result: *const c_char, error: *const bw_Err ) where
	H: FnOnce(BrowserThreaded, Result<String, JsEvaluationError>) + Send
{
	let data_ptr = cb_data as *mut H;
	let data = unsafe { Box::from_raw( data_ptr ) };

	let (handle, result) = ffi_eval_js_callback_result( bw, _result, error );

	(*data)( handle.into(), result );
}
