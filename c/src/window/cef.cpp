#include "../window.h"

#include "../cef/util.hpp"
#include "../common.h"

#include <include/cef_base.h>
#include <include/views/cef_window.h>



class MyWindowDelegate : public CefWindowDelegate {
	bw_WindowOptions options;

public:
	MyWindowDelegate( const bw_WindowOptions& options ) : options(options) {}

	bool CanClose( CefRefPtr<CefWindow> window ) override {
		UNUSED( window );
		return true;
	}

	bool CanMaximize( CefRefPtr<CefWindow> window ) override {
		UNUSED( window );
		return true;
	}

	bool CanMinimize( CefRefPtr<CefWindow> window ) override {
		UNUSED( window );
		return this->options.minimizable;
	}

	bool CanResize( CefRefPtr<CefWindow> window ) override {
		UNUSED( window );
		return this->options.resizable;
	}

	CefRefPtr<CefWindow> GetParentWindow( CefRefPtr<CefWindow> window, bool* is_menu, bool* can_activate_menu ) override {
		UNUSED( window );
		UNUSED( is_menu );
		UNUSED( can_activate_menu );
		return NULL;
	}

	bool IsFrameless( CefRefPtr<CefWindow> window ) override {
		UNUSED( window );
		return !this->options.borders;
	}

	bool OnAccelerator( CefRefPtr<CefWindow> window, int command_id ) override {
		UNUSED( window );
		UNUSED( command_id );
		return false;
	}

	bool OnKeyEvent( CefRefPtr<CefWindow> window, const CefKeyEvent& event ) override {
		UNUSED( window );
		return false;
	}

	void OnWindowCreated( CefRefPtr<CefWindow> window ) override {
		UNUSED( window );
	}

	void OnWindowDestroyed( CefRefPtr<CefWindow> window ) override {
		UNUSED( window );
	}

protected:
	IMPLEMENT_REFCOUNTING(MyWindowDelegate);
};



// Opacity is not supported with CEF's window API.
uint8_t bw_Window_getOpacity( bw_Window* window ) {
	return 255;
}

void bw_Window_setOpacity( bw_Window* window, uint8_t opacity ) {
	UNUSED( window );
	UNUSED( opacity );
}

bw_WindowImpl bw_WindowImpl_new(
	const bw_Window* _window,
	bw_CStrSlice _title,
	int width, int height,
	const bw_WindowOptions* options
) {
	UNUSED( _window );

	CefRefPtr<CefWindowDelegate> cef_window_options( new MyWindowDelegate( *options ) );
	CefRefPtr<CefWindow> window = CefWindow::CreateTopLevelWindow( cef_window_options );

	window->SetTitle( bw_cef_copyToString( _title ) );

	CefSize size( width, height );
	window->SetSize( size );

	bw_WindowImpl impl;
	impl.handle_ptr = new CefRefPtr<CefWindow>( window );
	return impl;
}

void bw_WindowImpl_destroy( bw_WindowImpl* window ) {
	delete (CefRefPtr<CefWindow>*)window->handle_ptr;
}

void bw_WindowImpl_hide( bw_WindowImpl* window ) {
	(*(CefRefPtr<CefWindow>*)window->handle_ptr)->Hide();
}

void bw_WindowImpl_show( bw_WindowImpl* window ) {
	(*(CefRefPtr<CefWindow>*)window->handle_ptr)->Show();
}



bw_Dims2D bw_Window_getContentDimensions( bw_Window* window ) { printf("bw_Window_getContentDimensions\n");
	CefRect rect = (*(CefRefPtr<CefWindow>*)window->impl.handle_ptr)->GetClientAreaBoundsInScreen();

	bw_Dims2D dims;
	dims.width = rect.width;
	dims.height = rect.height;

	return dims;
}

bw_Pos2D bw_Window_getPosition( bw_Window* window ) { printf("bw_Window_getPosition\n");
	CefRect rect = (*(CefRefPtr<CefWindow>*)window->impl.handle_ptr)->GetClientAreaBoundsInScreen();

	bw_Pos2D pos;
	pos.x = rect.x;
	pos.y = rect.y;

	return pos;
}

size_t bw_Window_getTitle( bw_Window* window, char** title ) { printf("bw_Window_getTitle\n");

	CefString cef_title = (*(CefRefPtr<CefWindow>*)window->impl.handle_ptr)->GetTitle();

	return bw_cef_copyToCstr( cef_title, title );
}

bw_Dims2D bw_Window_getWindowDimensions( bw_Window* window ) {
	bw_Dims2D dims = { 0, 0 };
	return dims;
}

void bw_Window_setContentDimensions( bw_Window* window, bw_Dims2D dimensions ) {
	// Not supported...
	UNUSED(window)
	UNUSED(dimensions)
}

void bw_Window_setPosition( bw_Window* window, bw_Pos2D position ) { printf("bw_Window_setPosition\n");
	UNUSED(window)
	UNUSED(position)
}

void bw_Window_setTitle( bw_Window* window, bw_CStrSlice _title ) { printf("bw_Window_setTitle\n");
	CefString title = bw_cef_copyToString( _title );

	(*(CefRefPtr<CefWindow>*)window->impl.handle_ptr)->SetTitle(title);
}

void bw_Window_setWindowDimensions( bw_Window* window, bw_Dims2D dimensions ) { printf("bw_Window_setWindowDimensions\n");
	UNUSED(window)
	UNUSED(dimensions)
}