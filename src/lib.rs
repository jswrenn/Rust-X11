#![crate_name = "X11"]
#![crate_type = "rlib"]
#![feature(macro_rules, phase, unsafe_destructor)]
#![allow(raw_pointer_deriving)]

#[phase(plugin, link)]
extern crate log;
#[phase(plugin)]
extern crate new_type;
#[phase(plugin)]
extern crate refined_type;
#[phase(plugin)]
extern crate syntax_extensions;
extern crate libc;
extern crate serialize;

pub use connection_error::ConnectionError;
pub use connection_status::ConnectionStatus;
pub use window::Window;
pub use screen_setup::ScreenSetup;
pub use window_geometry::WindowGeometry;
pub use window_children::WindowChildren;
pub use window_attribute::WindowMainAttributeSet;
pub use window_attribute::WindowSubAttributeSet;
pub use window_attribute::WindowAttributeSet;

mod xcb;
pub mod input;

macro_rules! assert_type_eq(
    ($T1:ty, $T2:ty) => ( assert_type_eq!($T1, $T2, id) );
    ($T1:ty, $T2:ty, $ID:ident) => (
        #[allow(dead_code)]
        fn $ID(x: $T1) -> $T2 { x }
        );
)

///Represents a connection to an X server.
///Will automatically disconnect from the X server at end of object lifetime.
///Guaranteed to be a valid connection upon successful construction **but not after**.
#[deriving(Show)]
pub struct Connection {
    data: *mut xcb::xcb_connection_t
}

pub mod connection_error {
///This enum should represent a one-to-one mapping of the return values > 0 of
///xcb_connection_has_error.
//This enum should have identical size to an int in C to make it safe to cast
//the return value of xcb_connection_has_error into this enum.
assert_type_eq!(super::libc::c_int, i32)
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

pub mod connection_status {
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
    pub fn screen_setup(&'a self) -> ScreenSetup<'a> {
        ScreenSetup::new(self)
    }
}

pub type ScreenT = xcb::xcb_screen_t;
#[new_type]
pub struct Screen {
    data: ScreenT
}

impl Screen {
    #[inline]
    pub fn root_window(&self) -> Window {
        //FIXME Is this assumption justified?
        unsafe { Window::assume(self.data.root) }
    }
}

///An integral type capable of representing a `Window`.
pub type WindowInt = xcb::xcb_window_t;

refined_type!{
    use WindowInt;
    use xcb;

    #[deriving(Show, PartialEq, Eq, PartialOrd, Ord, Hash, Encodable, Decodable)]
    refined Window = WindowInt where |n: WindowInt| ->
        non_null <=> n != xcb::XCB_WINDOW_NONE,
        within_upper_bound <=> n <= (1 << 29) - 1
}

pub type RequestErrorT = *mut xcb::xcb_generic_error_t;
#[new_type]
pub struct RequestError {
    data: RequestErrorT
}

impl Drop for RequestError {
    fn drop(&mut self) {
        unsafe { libc::free(self.as_request_error_t() as *mut libc::c_void) }
    }
}

pub mod screen_setup {
use super::{std, xcb, Setup, Screen};

pub struct ScreenSetup<'a> {
    begin: xcb::xcb_screen_iterator_t,
    marker: std::kinds::marker::ContravariantLifetime<'a>
}

impl<'a> ScreenSetup<'a> {
    pub fn new(setup: &'a Setup) -> ScreenSetup<'a> {
        ScreenSetup { begin: unsafe { xcb::xcb_setup_roots_iterator(setup.data)  },
                      marker: std::kinds::marker::ContravariantLifetime }
    }
    
    pub fn iter(&'a self) -> Items<'a> {
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
            let mut error: *mut xcb::xcb_generic_error_t = std::ptr::null_mut();
            let reply = unsafe { $reply_func(self.connection.data, self.data, &mut error) };
            //If a reply is successfully received, the destructor for the cookie *must* not run.
            //If the destructor for the cookie ran, then the reply would be freed.
            unsafe { std::mem::forget(self) }
            if reply.is_null() {
                Err(RequestError::new(error))
            }
            else {
                debug_assert!(error.is_null(), "The pointer to the reply was nonnull but there is still a RequestError.")
                Ok($ReplyT::new(reply))
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

pub mod window_geometry {
    use super::{Connection, Window, RequestError, RequestDelay, Coordinate, RectangularSize, xcb, std, libc};

    #[deriving(Show)]
    pub struct Cookie<'a> {
        data: xcb::xcb_get_geometry_cookie_t,
        connection: &'a Connection
    }

    pub fn make_request<'a>(connection: &'a Connection, window: Window) -> (Cookie<'a>, RequestDelay<'a>) {
        let cookie = Cookie {
            data: unsafe { xcb::xcb_get_geometry(connection.data, window.as_window_int()) },
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

    pub type ReplyT = *mut xcb::xcb_get_geometry_reply_t;
    ///The reply from the X server holding the requested window's geometetrical information.
    #[new_type]
    pub struct Reply {
        data: ReplyT
    }

    impl Reply {
        ///All geometerical information of the requested Window.
        pub fn geometry(&self) -> WindowGeometry {
            let reply = unsafe { *self.as_reply_t() };
            let position = Coordinate { x: reply.x, y: reply.y };
            let size = RectangularSize { width: reply.width, height: reply.height };
            let border_width = reply.border_width;
            WindowGeometry::new(position, size, border_width)
        }
        ///The position of the requested window.
        pub fn position(&self) -> Coordinate {
            let reply = unsafe { *self.as_reply_t() };
            Coordinate { x: reply.x, y: reply.y }
        }

        ///The size of the requested window.
        pub fn size(&self) -> RectangularSize {
            let reply = unsafe { *self.as_reply_t() };
            RectangularSize { width: reply.width, height: reply.height }
        }

        ///The size of the border around the requested window.
        pub fn border_width(&self) -> u16 {
            unsafe { (*self.as_reply_t()).border_width }
        }
    }

    impl_reply_destructor!{}
    impl super::Reply for Reply {}
}

pub mod window_children {
use super::{Connection, Window, xcb, RequestDelay, RequestError, std, libc};

    #[deriving(Show)]
    pub struct Cookie<'a> {
        data: xcb::xcb_query_tree_cookie_t,
        connection: &'a Connection
    }

    pub fn make_request<'a>(connection: &'a Connection, window: Window) -> (Cookie<'a>, RequestDelay<'a>) {
        let cookie = Cookie {
            data: unsafe { xcb::xcb_query_tree(connection.data, window.as_window_int()) },
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

    pub type ReplyT = *mut xcb::xcb_query_tree_reply_t;
    #[new_type]
    pub struct Reply {
        data: ReplyT
    }

    impl Reply {
        pub fn children<'a>(&'a self) -> WindowChildren<'a> {
            unsafe {
                WindowChildren {
                    xs: std::mem::transmute(
                            std::raw::Slice {
                                data: xcb::xcb_query_tree_children(self.as_reply_t() as *const _) as *const xcb::xcb_window_t,
                                len: xcb::xcb_query_tree_children_length(self.as_reply_t() as *const _) as uint
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
        Connection::new_impl(std::ptr::null_mut())
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
            connection_status::Valid => {
                debug!("Connected to X server.")
                Ok(possible_connection)
            }
            connection_status::Invalid(error) => {
                debug!("Unable to connect to X server.")
                Err(error)
            }
        }
    }

    fn screen_of_display(&self, mut screen_number: libc::c_int) -> Option<Screen> {
        let setup = self.setup();
        let screen_setup = setup.screen_setup();
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
            0 => connection_status::Valid,
            n => {
                debug_assert!(n >= connection_error::Generic as libc::c_int && n <= connection_error::FDPassingFailure as libc::c_int, "A call to xcb_connection_has_error returned a value outside the expected range.")
                connection_status::Invalid(unsafe { std::mem::transmute(n) })
            }
        }
    }

    ///Test if connected to the X server.
    pub fn is_valid(&self) -> bool {
        match self.status() {
            connection_status::Valid => true,
            _                        => false
        }
    }

    pub fn flush(&self) {
        //FIXME: Failure is not handled yet.
        unsafe { xcb::xcb_flush(self.data) };
        debug!("Pending requests flushed.")
    }

    ///See: http://xcb.freedesktop.org/manual/group__XCB__Core__API.html#gafc379a27800bf735818a0760bd036321
    pub fn setup<'a>(&'a self) -> Setup<'a> {
        Setup {
            data: unsafe { xcb::xcb_get_setup(self.data) },
            marker: std::kinds::marker::ContravariantLifetime
        }
    }
     
    #[inline]
    pub fn make_window_geometry_request<'a>(&'a self, window: Window) -> (window_geometry::Cookie<'a>, RequestDelay<'a>) {
        window_geometry::make_request(self, window)
    }

    #[inline]
    pub fn make_window_children_request<'a>(&'a self, window: Window) -> (window_children::Cookie<'a>, RequestDelay<'a>) {
        window_children::make_request(self, window)
    }

    pub fn change_window_attributes<'a>(&'a self, window: Window, new_attributes: WindowAttributeSet) -> RequestDelay<'a> {
        let request_delay = RequestDelay::new(self);
        let main_attributes = new_attributes.main_attributes().bits();
        let sub_attributes = new_attributes.sub_attribute_array();
        unsafe {
            xcb::xcb_change_window_attributes(self.data, window.as_window_int(), main_attributes, std::mem::transmute(&sub_attributes));
        }
        request_delay
    }

    pub fn grab_key_chord<'a>(&'a self,
                              pointer_event_mode: input::PointerEventMode,
                              grab_window: Window,
                              key_with_modkey_set: input::KeyWithModkeySet,
                              pointer_mode: input::PointerMode,
                              keyboard_mode: input::KeyboardMode
                             ) -> RequestDelay<'a> {
        let request_delay = RequestDelay::new(self);
        unsafe {
            xcb::xcb_grab_key(self.data,
                              pointer_event_mode as input::pointer_event_mode::PointerEventModeInt,
                              grab_window.as_window_int(),
                              key_with_modkey_set.modkey_set.bits(),
                              key_with_modkey_set.keycode.as_keycode_t(),
                              pointer_mode as input::pointer_mode::PointerModeInt,
                              keyboard_mode as input::keyboard_mode::KeyboardModeInt
                             );
        }
        request_delay
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

pub mod window_attribute {
use xcb;
pub use self::background_pixmap::BackgroundPixmap;
pub use self::bit_gravity::BitGravity;
pub use self::win_gravity::WinGravity;
pub use self::backing_store::BackingStore;
pub use self::event::EventSet;
pub use self::colormap::ColormapSet;
pub use self::cursor::CursorSet;

///See documentation for WindowAttributeSet.
#[deriving(Show)]
pub struct WindowSubAttributeSet {
    pub background_pixmap: BackgroundPixmap,
    pub bit_gravity:       BitGravity,
    pub win_gravity:       WinGravity,
    pub backing_store:     BackingStore,
    pub event_set:         EventSet,
    pub colormap_set:      ColormapSet,
    pub cursor_set:        CursorSet
}

impl WindowSubAttributeSet {
    #[inline]
    pub fn new(background_pixmap_: BackgroundPixmap,
               bit_gravity_:       BitGravity,
               win_gravity_:       WinGravity,
               backing_store_:     BackingStore,
               event_set_:         EventSet,
               colormap_set_:      ColormapSet,
               cursor_set_:        CursorSet
              ) -> WindowSubAttributeSet {
        WindowSubAttributeSet {
            background_pixmap: background_pixmap_,
            bit_gravity:     bit_gravity_,
            win_gravity:     win_gravity_,
            backing_store:   backing_store_,
            event_set:       event_set_,
            colormap_set:    colormap_set_,
            cursor_set:      cursor_set_
        }
    }

    #[allow(unused_assignments)]
    pub fn to_array_for_attr(&self, window_attributes: WindowMainAttributeSet) -> [u32, ..7] {
        let mut i = 0;
        let mut result: [u32, ..7] = [0, 0, 0, 0, 0, 0, 0];
        if window_attributes.intersects(BACKGROUND_PIXMAP) {
            result[i] = self.background_pixmap.as_u32();
            i += 1;
        }
        if window_attributes.intersects(BIT_GRAVITY) {
            result[i] = self.bit_gravity as u32;
            i += 1;
        }
        if window_attributes.intersects(WIN_GRAVITY) {
            result[i] = self.win_gravity as u32;
            i += 1;
        }
        if window_attributes.intersects(BACKING_STORE) {
            result[i] = self.backing_store as u32;
            i += 1;
        }
        if window_attributes.intersects(EVENT) {
            result[i] = self.event_set.bits();
            i += 1;
        }
        if window_attributes.intersects(COLORMAP) {
            result[i] = self.colormap_set.bits();
            i += 1;
        }
        if window_attributes.intersects(CURSOR) {
            result[i] = self.cursor_set.bits();
            i += 1;
        }
        result
    }
}

///A bitmask of 15 flags where the flags are `back_pixmap`, `back_pixel`, ... `cursor`.
pub type WindowMainAttributeInt = xcb::xcb_cw_t;
bitflags!{
    #[deriving(Show)] flags WindowMainAttributeSet: WindowMainAttributeInt {
        const BACKGROUND_PIXMAP  = xcb::XCB_CW_BACK_PIXMAP,
        const BACKGROUND_PIXEL   = xcb::XCB_CW_BACK_PIXEL,
        const BORDER_PIXMAP      = xcb::XCB_CW_BORDER_PIXMAP,
        const BORDER_PIXEL       = xcb::XCB_CW_BORDER_PIXEL,
        const BIT_GRAVITY        = xcb::XCB_CW_BIT_GRAVITY,
        const WIN_GRAVITY        = xcb::XCB_CW_WIN_GRAVITY,
        const BACKING_STORE      = xcb::XCB_CW_BACKING_STORE,
        const BACKING_PLANES     = xcb::XCB_CW_BACKING_PLANES,
        const BACKING_PIXEL      = xcb::XCB_CW_BACKING_PIXEL,
        const OVERRIDE_REDDIRECT = xcb::XCB_CW_OVERRIDE_REDIRECT,
        const SAVE_UNDER         = xcb::XCB_CW_SAVE_UNDER,
        const EVENT              = xcb::XCB_CW_EVENT_MASK,
        const DONT_PROPAGATE     = xcb::XCB_CW_DONT_PROPAGATE,
        const COLORMAP           = xcb::XCB_CW_COLORMAP,
        const CURSOR             = xcb::XCB_CW_CURSOR
    }
}

///A `struct` which can be modeled as an algebraic datatype consisting of a
///unique data constructor for each element of the 15-ary Cartesian product
///of `back_pixmap` × `back_pixel` × ... × `cursor` where `back_pixmap`,
///`back_pixel`, ... and `cursor` are modeled as sets each equal to
///{ “on”, “off” }. (i.e.
/// ```
///data WindowAttributeSet = back_pixmap_on__back_pixel_on__...__cursor_on
///                        | back_pixmap_off__back_pixel_on__...__cursor_on
///                        | back_pixmap_off__back_pixel_off__...__cursor_on
///                        ...
///                        -- Obviously listing 2¹⁵ data constructors is impractical.
/// ```
///)
///Indicate the data constructor one intends to use via the `main_attributes`
///field.
///Additionally, some of the data constructors (specifically the ones indicating
///that `back_pixmap`, `bit_gravity`, `win_gravity`, `backing_store`, `event`,
///`colormap`, or `cursor` are “on”) take arguments.
///Indicate the arguments to the data constructor one intends to use via the
///`sub_attributes` field.
#[deriving(Show)]
pub struct WindowAttributeSet {
    pub main_attributes: WindowMainAttributeSet,
    pub sub_attributes: WindowSubAttributeSet
}

impl WindowAttributeSet {
    #[inline]
    pub fn new(main_attributes: WindowMainAttributeSet,
               sub_attributes: WindowSubAttributeSet
              ) -> WindowAttributeSet {
        WindowAttributeSet { main_attributes: main_attributes, sub_attributes: sub_attributes }
    }
    #[inline]
    pub fn main_attributes(&self) -> WindowMainAttributeSet { self.main_attributes }
    #[inline]
    pub fn sub_attributes(&self) -> WindowSubAttributeSet { self.sub_attributes }
    #[inline]
    pub fn sub_attribute_array(&self) -> [u32, ..7] {
        self.sub_attributes.to_array_for_attr(self.main_attributes)
    }
}

pub mod background_pixmap {
    use std;
    use xcb;

    #[deriving(Show, PartialEq, Eq)]
    pub struct BackgroundPixmap {
        data: u32
    }

    static NONE: BackgroundPixmap = BackgroundPixmap { data: xcb::XCB_BACK_PIXMAP_NONE };
    static PARENT_RELATIVE: BackgroundPixmap = BackgroundPixmap { data: xcb::XCB_BACK_PIXMAP_PARENT_RELATIVE };

    impl BackgroundPixmap {
        #[inline]
        pub fn new(n: u32) -> BackgroundPixmap {
            BackgroundPixmap { data: n }
        }
        #[inline]
        pub fn is_pixmap(&self) -> bool { (*self != self::NONE) && (*self != self::PARENT_RELATIVE) }
        #[inline]
        pub fn as_u32(&self) -> u32 { self.data }
    }

    impl std::default::Default for BackgroundPixmap {
        #[inline]
        fn default() -> BackgroundPixmap { self::NONE }
    }

}

pub mod border_pixmap {
    use std;
    use xcb;

    #[deriving(Show, PartialEq, Eq)]
    pub struct BorderPixmap {
        data: u32
    }

    static COPY_FROM_PARENT: BorderPixmap = BorderPixmap { data: xcb::XCB_COPY_FROM_PARENT };

    impl BorderPixmap {
        #[inline]
        pub fn new(n: u32) -> BorderPixmap { BorderPixmap { data: n } }
        #[inline]
        pub fn is_pixmap(&self) -> bool { *self != self::COPY_FROM_PARENT }
        #[inline]
        pub fn as_u32(&self) -> u32 { self.data }
    }

    impl std::default::Default for BorderPixmap {
        #[inline]
        fn default() -> BorderPixmap { self::COPY_FROM_PARENT }
    }

}

pub mod bit_gravity {
    use xcb;
    pub type BitGravityInt = xcb::xcb_gravity_t;
    assert_type_eq!(BitGravityInt, u32)
    #[repr(u32)]
    #[deriving(Show)]
    pub enum BitGravity {
        BitForget  = xcb::XCB_GRAVITY_BIT_FORGET,
        NorthWest  = xcb::XCB_GRAVITY_NORTH_WEST,
        North      = xcb::XCB_GRAVITY_NORTH,
        NorthEast  = xcb::XCB_GRAVITY_NORTH_EAST,
        West       = xcb::XCB_GRAVITY_WEST,
        Center     = xcb::XCB_GRAVITY_CENTER,
        East       = xcb::XCB_GRAVITY_EAST,
        SouthWest  = xcb::XCB_GRAVITY_SOUTH_WEST,
        South      = xcb::XCB_GRAVITY_SOUTH,
        Static     = xcb::XCB_GRAVITY_STATIC
    }
}

pub mod win_gravity {
    use xcb;
    pub type WinGravityInt = xcb::xcb_gravity_t;
    assert_type_eq!(WinGravityInt, u32)
    #[repr(u32)]
    #[deriving(Show)]
    pub enum WinGravity {
        WinUnmap   = xcb::XCB_GRAVITY_WIN_UNMAP,
        NorthWest  = xcb::XCB_GRAVITY_NORTH_WEST,
        North      = xcb::XCB_GRAVITY_NORTH,
        NorthEast  = xcb::XCB_GRAVITY_NORTH_EAST,
        West       = xcb::XCB_GRAVITY_WEST,
        Center     = xcb::XCB_GRAVITY_CENTER,
        East       = xcb::XCB_GRAVITY_EAST,
        SouthWest  = xcb::XCB_GRAVITY_SOUTH_WEST,
        South      = xcb::XCB_GRAVITY_SOUTH,
        Static     = xcb::XCB_GRAVITY_STATIC
    }
}

pub mod backing_store {
    use xcb;
    pub type BackingStoreInt = xcb::xcb_backing_store_t;
    assert_type_eq!(BackingStoreInt, u32)
    #[repr(u32)]
    #[deriving(Show)]
    pub enum BackingStore {
        NotUseful  = xcb::XCB_BACKING_STORE_NOT_USEFUL,
        WhenMapped = xcb::XCB_BACKING_STORE_WHEN_MAPPED,
        Always     = xcb::XCB_BACKING_STORE_ALWAYS
    }
}

pub mod event {
    use xcb;
    pub type EventInt = xcb::xcb_event_mask_t;
    bitflags!{
        #[deriving(Show)] flags EventSet: EventInt {
            const NO_EVENT              = xcb::XCB_EVENT_MASK_NO_EVENT,
            const KEY_PRESS             = xcb::XCB_EVENT_MASK_KEY_PRESS,
            const KEY_RELEASE           = xcb::XCB_EVENT_MASK_KEY_RELEASE,
            const BUTTON_PRESS          = xcb::XCB_EVENT_MASK_BUTTON_PRESS,
            const BUTTON_RELEASE        = xcb::XCB_EVENT_MASK_BUTTON_RELEASE,
            const ENTER_WINDOW          = xcb::XCB_EVENT_MASK_ENTER_WINDOW,
            const LEAVE_WINDOW          = xcb::XCB_EVENT_MASK_LEAVE_WINDOW,
            const POINTER_MOTION        = xcb::XCB_EVENT_MASK_POINTER_MOTION,
            const MOTION_HINT           = xcb::XCB_EVENT_MASK_POINTER_MOTION_HINT,
            const BUTTON_1_MOTION       = xcb::XCB_EVENT_MASK_BUTTON_1_MOTION,
            const BUTTON_2_MOTION       = xcb::XCB_EVENT_MASK_BUTTON_2_MOTION,
            const BUTTON_3_MOTION       = xcb::XCB_EVENT_MASK_BUTTON_3_MOTION,
            const BUTTON_4_MOTION       = xcb::XCB_EVENT_MASK_BUTTON_4_MOTION,
            const BUTTON_5_MOTION       = xcb::XCB_EVENT_MASK_BUTTON_5_MOTION,
            const BUTTON_MOTION         = xcb::XCB_EVENT_MASK_BUTTON_MOTION,
            const KEYMAP_STATE          = xcb::XCB_EVENT_MASK_KEYMAP_STATE,
            const EXPOSURE              = xcb::XCB_EVENT_MASK_EXPOSURE,
            const VISIBILITY_CHANGE     = xcb::XCB_EVENT_MASK_VISIBILITY_CHANGE,
            const STRUCTURE_NOTIFY      = xcb::XCB_EVENT_MASK_STRUCTURE_NOTIFY,
            const RESIZE_REDIRECT       = xcb::XCB_EVENT_MASK_RESIZE_REDIRECT,
            const SUBSTRUCTURE_NOTIFY   = xcb::XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY,
            const SUBSTRUCTURE_REDIRECT = xcb::XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT,
            const FOCUS_CHANGE          = xcb::XCB_EVENT_MASK_FOCUS_CHANGE,
            const PROPERTY_CHANGE       = xcb::XCB_EVENT_MASK_PROPERTY_CHANGE,
            const COLOR_MAP_CHANGE      = xcb::XCB_EVENT_MASK_COLOR_MAP_CHANGE,
            const OWNER_GRAP_BUTTON     = xcb::XCB_EVENT_MASK_OWNER_GRAB_BUTTON
        }
    }
}

pub mod colormap {
    use xcb;
    pub type ColormapInt = xcb::xcb_colormap_enum_t;
    bitflags!{
        #[deriving(Show)] flags ColormapSet: ColormapInt {
            const NONE = xcb::XCB_COLORMAP_NONE
        }
    }
}

pub mod cursor {
    use xcb;
    pub type CursorInt = xcb::xcb_cursor_enum_t;
    bitflags!{
        #[deriving(Show)] flags CursorSet: CursorInt {
            const NONE = xcb::XCB_CURSOR_NONE
        }
    }
}

}
