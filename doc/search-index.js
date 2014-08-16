var searchIndex = {};
searchIndex['X11'] = {"items":[[0,"","X11",""],[1,"Connection","","Represents a connection to an X server.\nWill automatically disconnect from the X server at end of object lifetime.\nGuaranteed to be a valid connection upon successful construction **but not after**."],[1,"Setup","","See: http://xcb.freedesktop.org/manual/group__XCB__Core__API.html#gafc379a27800bf735818a0760bd036321"],[1,"Screen","",""],[1,"Window","",""],[1,"RequestError","",""],[1,"RequestDelay","","Represents a request that is waiting to be sent.\nFlushes all pending requests upon destruction."],[1,"Coordinate","","A 2D coordinate where (0, 0) is the upper left corner."],[11,"x","","",0],[11,"y","","",0],[1,"RectangularSize","","A simple representation for the size of a rectangle."],[11,"width","","",1],[11,"height","","",1],[0,"Connection_Error","",""],[2,"ConnectionError","X11::Connection_Error","This enum should represent a one-to-one mapping of the return values > 0 of\nxcb_connection_has_error."],[12,"Generic","","Socket error, pipe error, or other stream error",2],[12,"ExtNotSupported","","Extension not supported",2],[12,"MemInsufficient","","Memory not available",2],[12,"ReqLenExceeded","","Exceeding request length for server",2],[12,"ParseErr","","Unable to parse display string",2],[12,"InvalidScreen","","No screen matching display on server\n(The display is usually specified with the $DISPLAY environment variable.)",2],[12,"FDPassingFailure","","File descriptor passing operation failure\n(This is not explicitly stated as a possible return value in the comments\nabove the declaration of xcb_connection_has_error in xcb.h, but it is\nimplied as a possible return value from the macro definition\nXCB_CONN_CLOSED_FDPASSING_FAILED.)",2],[10,"rand","","",2],[10,"eq","","",2],[10,"ne","","",2],[10,"fmt","","",2],[0,"Connection_Status","X11",""],[2,"ConnectionStatus","X11::Connection_Status",""],[12,"Valid","","",3],[12,"Invalid","","",3],[0,"Screen_Setup","X11",""],[1,"ScreenSetup","X11::Screen_Setup",""],[1,"Items","",""],[10,"new","","",4],[10,"iter","","",4],[10,"next","","",5],[0,"Window_Geometry","X11",""],[1,"Cookie","X11::Window_Geometry",""],[1,"WindowGeometry","","Holds the position, size, and border width of a window."],[1,"Reply","","The reply from the X server holding the requested window's geometetrical information."],[3,"make_request","",""],[10,"fmt","","",6],[10,"wait_for_reply","","",6],[10,"drop","","",6],[10,"fmt","","",7],[10,"new","","",7],[10,"position","","",7],[10,"size","","",7],[10,"border_width","","",7],[10,"geometry","","All geometerical information of the requested Window.",8],[10,"position","","The position of the requested window.",8],[10,"size","","The size of the requested window.",8],[10,"border_width","","The size of the border around the requested window.",8],[10,"drop","","",8],[0,"Window_Children","X11",""],[1,"Cookie","X11::Window_Children",""],[1,"Reply","",""],[1,"WindowChildren","",""],[1,"Items","",""],[3,"make_request","",""],[10,"fmt","","",9],[10,"wait_for_reply","","",9],[10,"drop","","",9],[10,"children","","",10],[10,"drop","","",10],[10,"iter","","",11],[10,"next","","",12],[6,"Cookie","X11","Implementors of this trait represent a pending reply from the X server made via an asynchronous\nrequest."],[9,"wait_for_reply","","",13],[6,"Reply","",""],[10,"fmt","","",14],[10,"screen_setup","","",15],[10,"fmt","","",16],[10,"fmt","","",17],[10,"id","","",17],[10,"root_window","","",16],[10,"drop","","",18],[10,"new","","",19],[10,"force","","Use force to flush all pending requests.\nThe RequestDelay called with the force method is moved into\nthe force method where its destructor is called (once).",19],[10,"subsume","","Use subsume to prevent *other* RequestDelays in the current scope\nfrom calling their destructors (and flushing pending requests).\nThe RequestDelay placed in the “other” parameter\nis moved into the subsume method where its destructor is made to do\nnothing.  No error can occur if subsume is not used, but it helps\ncontrol exactly when pending requests are flushed.",19],[10,"drop","","",19],[10,"new","","Construct a connection to the X server only if it can be shown to be an initially valid\nconnection.\nConnects to the X server specified by the $DISPLAY environment variable (if $DISPLAY\ncan be parsed).",14],[10,"new_with_default_screen","","Does the same as new() but also returns the default Screen if one exists.",14],[10,"status","","Test if connected to the X server and if not return why.",14],[10,"is_valid","","Test if connected to the X server.",14],[10,"flush","","",14],[10,"setup","","See: http://xcb.freedesktop.org/manual/group__XCB__Core__API.html#gafc379a27800bf735818a0760bd036321",14],[10,"make_window_geometry_request","","",14],[10,"make_window_children_request","","",14],[10,"drop","","Will disconnect the Connection from the X server.",14],[10,"fmt","","",0],[10,"fmt","","",1]],"paths":[[1,"Coordinate"],[1,"RectangularSize"],[2,"ConnectionError"],[2,"ConnectionStatus"],[1,"ScreenSetup"],[1,"Items"],[1,"Cookie"],[1,"WindowGeometry"],[1,"Reply"],[1,"Cookie"],[1,"Reply"],[1,"WindowChildren"],[1,"Items"],[6,"Cookie"],[1,"Connection"],[1,"Setup"],[1,"Screen"],[1,"Window"],[1,"RequestError"],[1,"RequestDelay"]]};
initSearch(searchIndex);
