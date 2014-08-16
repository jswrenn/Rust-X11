#![crate_name = "X11"]
#![crate_type = "rlib"]
#![feature(macro_rules, phase, unsafe_destructor)]
#![allow(raw_pointer_deriving)]
#[phase(plugin, link)] extern crate log;
extern crate libc;
pub use Connection_Error::ConnectionError;
pub use Connection_Status::ConnectionStatus;
pub use Screen_Setup::ScreenSetup;
pub use Window_Geometry::WindowGeometry;
pub use Window_Children::WindowChildren;

mod xcb;

///Represents a connection to an X server.
///Will automatically disconnect from the X server at end of object lifetime.
///Guaranteed to be a valid connection upon successful construction **but not after**.
#[deriving(Show)]
pub struct Connection {
    data: *mut xcb::xcb_connection_t
}

pub mod Connection_Error {
//FIXME repr() won't take c_int, so find a way to make sure i32 = c_int.
///This enum should represent a one-to-one mapping of the return values > 0 of
///xcb_connection_has_error.
//This enum should have identical size to an int in C to make it safe to cast
//the return value of xcb_connection_has_error into this enum.
#[repr(i32)]
#[deriving(Show, PartialEq, Eq, Rand)]
pub enum ConnectionError {
    ///Socket error, pipe error, or other stream error
    Generic = 1,
    ///Extension not supported
    ExtNotSupported = 2,
    ///Memory not available
    MemInsufficient = 3,
    ///Exceeding request length for server
    ReqLenExceeded = 4,
    ///Unable to parse display string
    ParseErr = 5,
    ///No screen matching display on server
    ///(The display is usually specified with the $DISPLAY environment variable.)
    //FIXME Needs to mention explicitly specifying the display string without
    //an environment variable when the interface is added.
    InvalidScreen = 6,
    ///File descriptor passing operation failure
    ///(This is not explicitly stated as a possible return value in the comments
    ///above the declaration of xcb_connection_has_error in xcb.h, but it is
    ///implied as a possible return value from the macro definition
    ///XCB_CONN_CLOSED_FDPASSING_FAILED.)
    FDPassingFailure = 7,
}
}

pub mod Connection_Status {
pub enum ConnectionStatus {
    Valid,
    Invalid(::ConnectionError)
}
}

///See: http://xcb.freedesktop.org/manual/group__XCB__Core__API.html#gafc379a27800bf735818a0760bd036321
pub struct Setup<'a> {
    data: *const xcb::xcb_setup_t,
    marker: std::kinds::marker::ContravariantLifetime<'a>
}

impl<'a> Setup<'a> {
    pub fn new<'a>(connection: &'a Connection) -> Setup<'a> {
        Setup { data: unsafe { xcb::xcb_get_setup(connection.data) },
                marker: std::kinds::marker::ContravariantLifetime }
    }
}

//This is like a Haskell newtype.
//Adding/removing/changing fields invalidates code which transmutes between the underlying type
//and the struct.
#[deriving(Show)]
pub struct Screen {
    data: xcb::xcb_screen_t
}

//This is like a Haskell newtype.
//Adding/removing/changing fields invalidates code which transmutes between the underlying type
//and the struct.
#[deriving(Show)]
pub struct Window {
    data: xcb::xcb_window_t
}

impl Window {
    pub fn get_id(&self) -> u32 { self.data }
}

impl Screen {
    pub fn get_root_window(&self) -> Window {
        Window { data: self.data.root }
    }
}

pub struct RequestError {
    data: *mut xcb::xcb_generic_error_t
}

impl Drop for RequestError {
    fn drop(&mut self) {
        unsafe { libc::free(self.data as *mut libc::c_void) }
    }
}

pub mod Screen_Setup {
use super::{std, xcb, Setup, Screen};

pub struct ScreenSetup<'a> {
    begin: xcb::xcb_screen_iterator_t,
    marker: std::kinds::marker::ContravariantLifetime<'a>
}

impl<'a> ScreenSetup<'a> {
    pub fn new<'a>(setup: &'a Setup) -> ScreenSetup<'a> {
        ScreenSetup { begin: unsafe { xcb::xcb_setup_roots_iterator(setup.data)  },
                      marker: std::kinds::marker::ContravariantLifetime }
    }
    
    pub fn iter<'a>(&'a self) -> Items<'a> {
        Items {
            current: self.begin,
            marker: self.marker
        }
    }
}

pub struct Items<'a> {
    current: xcb::xcb_screen_iterator_t,
    marker: std::kinds::marker::ContravariantLifetime<'a>
}

impl<'a> Iterator<&'a Screen> for Items<'a> {
    fn next(&mut self) -> Option<&'a Screen> {
        match self.current.rem {
            0 => None,
            _ => {
                let current_screen =  unsafe { std::mem::transmute(self.current.data) };
                unsafe { xcb::xcb_screen_next(&mut self.current) }
                Some(current_screen)
            }
        }
    }
}

}

///Implementors of this trait represent a pending reply from the X server made via an asynchronous
///request.
pub trait Cookie<'a, R: Reply>: Drop {
    fn wait_for_reply(self) -> Result<R, RequestError>;
}

pub trait Reply: Drop {}

macro_rules! impl_wait_for_reply(
    ($reply_func:expr)                => (impl_wait_for_reply!($reply_func, Reply));
    ($reply_func:expr, $ReplyT:ident) => (
        fn wait_for_reply(self) -> Result<$ReplyT, RequestError> {
            let mut error: *mut xcb::xcb_generic_error_t = std::ptr::mut_null();
            let reply = unsafe { $reply_func(self.connection.data, self.data, &mut error) };
            //If a reply is successfully received, the destructor for the cookie *must* not run.
            //If the destructor for the cookie ran, then the reply would be freed.
            unsafe { std::mem::forget(self) }
            if reply.is_null() {
                Err(RequestError { data: error })
            }
            else {
                debug_assert!(error.is_null(), "The pointer to the reply was nonnull but there is still a RequestError.")
                Ok($ReplyT { data: reply })
            }
        }
    );
)

///Use this macro to implement the destructor for all implementors of the Cookie trait.
///This is for preventing a memory leak for when a reply is received, but it is
///impossible to retrieve (i.e. fail! was called in the task which owned
///the cookie necessary for retrieving the reply).
macro_rules! impl_cookie_destructor(
    ()               => (impl_cookie_destructor!(Cookie));
    ($CookieT:ident) => (
        #[unsafe_destructor]
        impl<'a> Drop for $CookieT<'a> {
            fn drop(&mut self) {
                let reply_number = self.data.sequence;
                unsafe { xcb::xcb_discard_reply(self.connection.data, reply_number) }
                debug!("Discarded reply number: {}", reply_number)
            }
        }
    );
)

macro_rules! impl_reply_destructor(
    ()              => (impl_reply_destructor!(Reply));
    ($ReplyT:ident) => (
        impl Drop for $ReplyT {
            fn drop(&mut self) {
                unsafe { libc::free(self.data as *mut libc::c_void) }
            }
        }
    );
)

///Represents a request that is waiting to be sent.
///Flushes all pending requests upon destruction.
pub struct RequestDelay<'a> {
    connection: &'a Connection
}

impl<'a> RequestDelay<'a> {
    pub fn new(connection: &'a Connection) -> RequestDelay<'a> {
        debug!("Pending request on connection: {}", connection)
        RequestDelay { connection: connection }
    }

    ///Use force to flush all pending requests.
    ///The RequestDelay called with the force method is moved into
    ///the force method where its destructor is called (once).
    pub fn force(self) {}
    ///Use subsume to prevent *other* RequestDelays in the current scope
    ///from calling their destructors (and flushing pending requests).
    ///The RequestDelay placed in the “other” parameter
    ///is moved into the subsume method where its destructor is made to do
    ///nothing.  No error can occur if subsume is not used, but it helps
    ///control exactly when pending requests are flushed.
    pub fn subsume(&self, other: RequestDelay) {
        unsafe { std::mem::forget(other) }
    }
}

#[unsafe_destructor]
impl<'a> Drop for RequestDelay<'a> {
    fn drop(&mut self) {
        self.connection.flush()
    }
}

pub mod Window_Geometry {
    use super::{Connection, Window, RequestError, RequestDelay, Coordinate, RectangularSize, xcb, std, libc};

    #[deriving(Show)]
    pub struct Cookie<'a> {
        data: xcb::xcb_get_geometry_cookie_t,
        connection: &'a Connection
    }

    pub fn make_request<'a>(connection: &'a Connection, window: Window) -> (Cookie<'a>, RequestDelay<'a>) {
        let cookie = Cookie {
            data: unsafe { xcb::xcb_get_geometry(connection.data, window.get_id()) },
            connection: connection
        };
        let request_delay = RequestDelay::new(connection);
        debug!("Pending window geometry request with cookie: {}", cookie);
        (cookie, request_delay)
    }

    impl<'a> super::Cookie<'a, Reply> for Cookie<'a> {
        impl_wait_for_reply!(xcb::xcb_get_geometry_reply)
    }

    impl_cookie_destructor!{}

    ///Holds the position, size, and border width of a window.
    #[deriving(Show)]
    pub struct WindowGeometry {
        position_: Coordinate,
        size_: RectangularSize,
        border_width_: u16
    }

    impl WindowGeometry {
        pub fn new(position: Coordinate, size: RectangularSize, border_width: u16) -> WindowGeometry {
            WindowGeometry { position_: position, size_: size, border_width_: border_width }
        }

        pub fn position(&self) -> Coordinate { self.position_ }
        pub fn size(&self) -> RectangularSize { self.size_ }
        pub fn border_width(&self) -> u16 { self.border_width_ }
    }

    ///The reply from the X server holding the requested window's geometetrical information.
    pub struct Reply {
        data: *mut xcb::xcb_get_geometry_reply_t,
    }

    impl Reply {
        ///All geometerical information of the requested Window.
        pub fn geometry(&self) -> WindowGeometry {
            let reply = unsafe { *self.data };
            let position = Coordinate { x: reply.x, y: reply.y };
            let size = RectangularSize { width: reply.width, height: reply.height };
            let border_width = reply.border_width;
            WindowGeometry::new(position, size, border_width)
        }
        ///The position of the requested window.
        pub fn position(&self) -> Coordinate {
            let reply = unsafe { *self.data };
            Coordinate { x: reply.x, y: reply.y }
        }

        ///The size of the requested window.
        pub fn size(&self) -> RectangularSize {
            let reply = unsafe { *self.data };
            RectangularSize { width: reply.width, height: reply.height }
        }

        ///The size of the border around the requested window.
        pub fn border_width(&self) -> u16 {
            unsafe { (*self.data).border_width }
        }
    }

    impl_reply_destructor!{}
    impl super::Reply for Reply {}
}

pub mod Window_Children {
use super::{Connection, Window, xcb, RequestDelay, RequestError, std, libc};

    #[deriving(Show)]
    pub struct Cookie<'a> {
        data: xcb::xcb_query_tree_cookie_t,
        connection: &'a Connection
    }

    pub fn make_request<'a>(connection: &'a Connection, window: Window) -> (Cookie<'a>, RequestDelay<'a>) {
        let cookie = Cookie {
            data: unsafe { xcb::xcb_query_tree(connection.data, window.get_id()) },
            connection: connection
        };
        let request_delay = RequestDelay::new(connection);
        debug!("Pending window children request with cookie: {}", cookie);
        (cookie, request_delay)
    }

    impl<'a> super::Cookie<'a, Reply> for Cookie<'a> {
        impl_wait_for_reply!(xcb::xcb_query_tree_reply)
    }

    impl_cookie_destructor!{}

    pub struct Reply {
        data: *mut xcb::xcb_query_tree_reply_t
    }

    impl Reply {
        pub fn children<'a>(&'a self) -> WindowChildren<'a> {
            unsafe {
                WindowChildren {
                    xs: std::mem::transmute(
                            std::raw::Slice {
                                data: xcb::xcb_query_tree_children(self.data as *const _) as *const xcb::xcb_window_t,
                                len: xcb::xcb_query_tree_children_length(self.data as *const _) as uint
                            }
                        )
                }
            }
        }
    }

    impl_reply_destructor!{}
    impl super::Reply for Reply{}

    pub struct WindowChildren<'a> {
        xs: &'a [xcb::xcb_window_t],
    }

    impl<'a> WindowChildren<'a> {
        pub fn iter<'a>(&'a self) -> Items<'a> {
            Items {
                current: self.xs.iter()
            }
        }
    }

    pub struct Items<'a> {
        current: std::slice::Items<'a, xcb::xcb_window_t>,
    }

    impl<'a> Iterator<&'a Window> for Items<'a> {
        fn next(&mut self) -> Option<&'a Window> {
            unsafe { std::mem::transmute(self.current.next()) }
        }
    }
}

impl Connection {
    ///Construct a connection to the X server only if it can be shown to be an initially valid
    ///connection.
    ///Connects to the X server specified by the $DISPLAY environment variable (if $DISPLAY
    ///can be parsed).
    pub fn new() -> Result<Connection, ConnectionError> {
        Connection::new_impl(std::ptr::mut_null())
    }

    ///Does the same as new() but also returns the default Screen if one exists.
    pub fn new_with_default_screen() -> (Result<Connection, ConnectionError>, Option<Screen>) {
        let mut default_screen_number: libc::c_int = 0;
        let possible_connection = Connection::new_impl(&mut default_screen_number);
        match possible_connection {
            Ok(connection) => {
                let possible_default_screen = connection.screen_of_display(default_screen_number);
                (Ok(connection), possible_default_screen)
            }
            Err(_) => (possible_connection, None)
        }
    }

    fn new_impl(default_screen_number: *mut libc::c_int) -> Result<Connection, ConnectionError> {
        debug!("Connecting to X server...");
        let possible_connection_ptr = unsafe { xcb::xcb_connect(std::ptr::null(), default_screen_number) };
        //XCB guarantees that xcb_connect will not return a null pointer in its documentation.
        //(I was suspicious of this claim, so I even checked the source.)
        debug_assert!(!possible_connection_ptr.is_null(), "Null pointer returned by call to xcb_connect. This should be impossible.")
        let possible_connection = Connection { data: unsafe { &mut *possible_connection_ptr } };
        match possible_connection.status() {
            Connection_Status::Valid => {
                debug!("Connected to X server.")
                Ok(possible_connection)
            }
            Connection_Status::Invalid(error) => {
                debug!("Unable to connect to X server.")
                Err(error)
            }
        }
    }

    fn screen_of_display(&self, mut screen_number: libc::c_int) -> Option<Screen> {
        let setup = Setup::new(self);
        let screen_setup = ScreenSetup::new(&setup);
        for screen in screen_setup.iter() {
            if screen_number == 0 {
                return Some(*screen)
            }
            screen_number -= 1
        }
        None
    }

    ///Test if connected to the X server and if not return why.
    pub fn status(&self) -> ConnectionStatus {
        let connection_status = unsafe { xcb::xcb_connection_has_error(self.data) };
        match connection_status {
            0 => Connection_Status::Valid,
            n => {
                debug_assert!(n >= Connection_Error::Generic as libc::c_int && n <= Connection_Error::FDPassingFailure as libc::c_int, "A call to xcb_connection_has_error returned a value outside the expected range.")
                Connection_Status::Invalid(unsafe { std::mem::transmute(n) })
            }
        }
    }

    ///Test if connected to the X server.
    pub fn is_valid(&self) -> bool {
        match self.status() {
            Connection_Status::Valid => true,
            _                        => false
        }
    }

    pub fn flush(&self) {
        //FIXME: Failure is not handled yet.
        unsafe { xcb::xcb_flush(self.data) };
        debug!("Pending requests flushed.")
    }

    ///See: http://xcb.freedesktop.org/manual/group__XCB__Core__API.html#gafc379a27800bf735818a0760bd036321
    pub fn get_setup<'a>(&'a self) -> Setup<'a> {
        Setup {
            data: unsafe { xcb::xcb_get_setup(self.data) },
            marker: std::kinds::marker::ContravariantLifetime
        }
    }
     
    #[inline]
    pub fn make_window_geometry_request<'a>(&'a self, window: Window) -> (Window_Geometry::Cookie<'a>, RequestDelay<'a>) {
        Window_Geometry::make_request(self, window)
    }

    #[inline]
    pub fn make_window_children_request<'a>(&'a self, window: Window) -> (Window_Children::Cookie<'a>, RequestDelay<'a>) {
        Window_Children::make_request(self, window)
    }
}

impl Drop for Connection {
    ///Will disconnect the Connection from the X server.
    fn drop(&mut self) {
        unsafe { xcb::xcb_disconnect(self.data); }
        debug!("Disconnected from X server.")
    }
}

///A 2D coordinate where (0, 0) is the upper left corner.
#[deriving(Show)]
pub struct Coordinate {
    pub x: i16,
    pub y: i16
}

///A simple representation for the size of a rectangle.
#[deriving(Show)]
pub struct RectangularSize {
    pub width: u16,
    pub height: u16
}
