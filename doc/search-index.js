var searchIndex = {};
searchIndex['X11'] = {"items":[[0,"","X11",""],[1,"Connection","","Represents a connection to an X server.\nWill automatically disconnect from the X server at end of object lifetime.\nGuaranteed to be a valid connection upon successful construction **but not after**."],[1,"Setup","","See: http://xcb.freedesktop.org/manual/group__XCB__Core__API.html#gafc379a27800bf735818a0760bd036321"],[1,"Screen","",""],[1,"Window","",""],[1,"RequestError","",""],[1,"RequestDelay","","Represents a request that is waiting to be sent.\nFlushes all pending requests upon destruction."],[1,"Coordinate","","A 2D coordinate where (0, 0) is the upper left corner."],[11,"x","","",0],[11,"y","","",0],[1,"RectangularSize","","A simple representation for the size of a rectangle."],[11,"width","","",1],[11,"height","","",1],[0,"Connection_Error","",""],[2,"ConnectionError","X11::Connection_Error","This enum should represent a one-to-one mapping of the return values > 0 of\nxcb_connection_has_error."],[12,"Generic","","Socket error, pipe error, or other stream error",2],[12,"ExtNotSupported","","Extension not supported",2],[12,"MemInsufficient","","Memory not available",2],[12,"ReqLenExceeded","","Exceeding request length for server",2],[12,"ParseErr","","Unable to parse display string",2],[12,"InvalidScreen","","No screen matching display on server\n(The display is usually specified with the $DISPLAY environment variable.)",2],[12,"FDPassingFailure","","File descriptor passing operation failure\n(This is not explicitly stated as a possible return value in the comments\nabove the declaration of xcb_connection_has_error in xcb.h, but it is\nimplied as a possible return value from the macro definition\nXCB_CONN_CLOSED_FDPASSING_FAILED.)",2],[10,"rand","","",2],[10,"eq","","",2],[10,"ne","","",2],[10,"fmt","","",2],[0,"Connection_Status","X11",""],[2,"ConnectionStatus","X11::Connection_Status",""],[12,"Valid","","",3],[12,"Invalid","","",3],[0,"Screen_Setup","X11",""],[1,"ScreenSetup","X11::Screen_Setup",""],[1,"Items","",""],[10,"new","","",4],[10,"iter","","",4],[10,"next","","",5],[0,"Window_Geometry","X11",""],[1,"Cookie","X11::Window_Geometry",""],[1,"WindowGeometry","","Holds the position, size, and border width of a window."],[1,"Reply","","The reply from the X server holding the requested window's geometetrical information."],[3,"make_request","",""],[10,"fmt","","",6],[10,"wait_for_reply","","",6],[10,"drop","","",6],[10,"fmt","","",7],[10,"new","","",7],[10,"position","","",7],[10,"size","","",7],[10,"border_width","","",7],[10,"geometry","","All geometerical information of the requested Window.",8],[10,"position","","The position of the requested window.",8],[10,"size","","The size of the requested window.",8],[10,"border_width","","The size of the border around the requested window.",8],[10,"drop","","",8],[0,"Window_Children","X11",""],[1,"Cookie","X11::Window_Children",""],[1,"Reply","",""],[1,"WindowChildren","",""],[1,"Items","",""],[3,"make_request","",""],[10,"fmt","","",9],[10,"wait_for_reply","","",9],[10,"drop","","",9],[10,"children","","",10],[10,"drop","","",10],[10,"iter","","",11],[10,"next","","",12],[0,"Window_Attribute","X11",""],[1,"WindowAttributeSet","X11::Window_Attribute",""],[0,"Back_Pixmap","",""],[1,"BackPixmapSet","X11::Window_Attribute::Back_Pixmap",""],[4,"BackPixmapInt","",""],[5,"none","",""],[5,"parent_relative","",""],[10,"hash","","",13],[10,"cmp","","",13],[10,"partial_cmp","","",13],[10,"lt","","",13],[10,"le","","",13],[10,"gt","","",13],[10,"ge","","",13],[10,"clone","","",13],[10,"eq","","",13],[10,"ne","","",13],[10,"fmt","","",13],[10,"empty","","Returns an empty set of flags.",13],[10,"all","","Returns the set containing all flags.",13],[10,"bits","","Returns the raw value of the flags currently stored.",13],[10,"from_bits","","Convert from underlying bit representation, unless that\nrepresentation contains bits that do not correspond to a flag.",13],[10,"from_bits_truncate","","Convert from underlying bit representation, dropping any bits\nthat do not correspond to flags.",13],[10,"is_empty","","Returns `true` if no flags are currently stored.",13],[10,"is_all","","Returns `true` if all flags are currently set.",13],[10,"intersects","","Returns `true` if there are flags common to both `self` and `other`.",13],[10,"contains","","Returns `true` all of the flags in `other` are contained within `self`.",13],[10,"insert","","Inserts the specified flags in-place.",13],[10,"remove","","Removes the specified flags in-place.",13],[10,"bitor","","Returns the union of the two sets of flags.",13],[10,"bitand","","Returns the intersection between the two sets of flags.",13],[10,"sub","","Returns the set difference of the two sets of flags.",13],[10,"not","","Returns the complement of this set of flags.",13],[0,"Backing_Store","X11::Window_Attribute",""],[1,"BackingStoreSet","X11::Window_Attribute::Backing_Store",""],[4,"BackingStoreInt","",""],[5,"not_useful","",""],[5,"when_mapped","",""],[5,"always","",""],[10,"hash","","",14],[10,"cmp","","",14],[10,"partial_cmp","","",14],[10,"lt","","",14],[10,"le","","",14],[10,"gt","","",14],[10,"ge","","",14],[10,"clone","","",14],[10,"eq","","",14],[10,"ne","","",14],[10,"fmt","","",14],[10,"empty","","Returns an empty set of flags.",14],[10,"all","","Returns the set containing all flags.",14],[10,"bits","","Returns the raw value of the flags currently stored.",14],[10,"from_bits","","Convert from underlying bit representation, unless that\nrepresentation contains bits that do not correspond to a flag.",14],[10,"from_bits_truncate","","Convert from underlying bit representation, dropping any bits\nthat do not correspond to flags.",14],[10,"is_empty","","Returns `true` if no flags are currently stored.",14],[10,"is_all","","Returns `true` if all flags are currently set.",14],[10,"intersects","","Returns `true` if there are flags common to both `self` and `other`.",14],[10,"contains","","Returns `true` all of the flags in `other` are contained within `self`.",14],[10,"insert","","Inserts the specified flags in-place.",14],[10,"remove","","Removes the specified flags in-place.",14],[10,"bitor","","Returns the union of the two sets of flags.",14],[10,"bitand","","Returns the intersection between the two sets of flags.",14],[10,"sub","","Returns the set difference of the two sets of flags.",14],[10,"not","","Returns the complement of this set of flags.",14],[0,"Event","X11::Window_Attribute",""],[1,"EventSet","X11::Window_Attribute::Event",""],[4,"EventInt","",""],[5,"no_event","",""],[5,"key_press","",""],[5,"key_release","",""],[5,"button_press","",""],[5,"button_release","",""],[5,"enter_window","",""],[5,"leave_window","",""],[5,"pointer_motion","",""],[5,"motion_hint","",""],[5,"button_1_motion","",""],[5,"button_2_motion","",""],[5,"button_3_motion","",""],[5,"button_4_motion","",""],[5,"button_5_motion","",""],[5,"button_motion","",""],[5,"keymap_state","",""],[5,"exposure","",""],[5,"visibility_change","",""],[5,"structure_notify","",""],[5,"resize_redirect","",""],[5,"substructure_notify","",""],[5,"substructure_redirect","",""],[5,"focus_change","",""],[5,"property_change","",""],[5,"color_map_change","",""],[5,"owner_grap_button","",""],[10,"hash","","",15],[10,"cmp","","",15],[10,"partial_cmp","","",15],[10,"lt","","",15],[10,"le","","",15],[10,"gt","","",15],[10,"ge","","",15],[10,"clone","","",15],[10,"eq","","",15],[10,"ne","","",15],[10,"fmt","","",15],[10,"empty","","Returns an empty set of flags.",15],[10,"all","","Returns the set containing all flags.",15],[10,"bits","","Returns the raw value of the flags currently stored.",15],[10,"from_bits","","Convert from underlying bit representation, unless that\nrepresentation contains bits that do not correspond to a flag.",15],[10,"from_bits_truncate","","Convert from underlying bit representation, dropping any bits\nthat do not correspond to flags.",15],[10,"is_empty","","Returns `true` if no flags are currently stored.",15],[10,"is_all","","Returns `true` if all flags are currently set.",15],[10,"intersects","","Returns `true` if there are flags common to both `self` and `other`.",15],[10,"contains","","Returns `true` all of the flags in `other` are contained within `self`.",15],[10,"insert","","Inserts the specified flags in-place.",15],[10,"remove","","Removes the specified flags in-place.",15],[10,"bitor","","Returns the union of the two sets of flags.",15],[10,"bitand","","Returns the intersection between the two sets of flags.",15],[10,"sub","","Returns the set difference of the two sets of flags.",15],[10,"not","","Returns the complement of this set of flags.",15],[0,"Colormap","X11::Window_Attribute",""],[1,"ColorMapSet","X11::Window_Attribute::Colormap",""],[4,"ColorMapInt","",""],[5,"none","",""],[10,"hash","","",16],[10,"cmp","","",16],[10,"partial_cmp","","",16],[10,"lt","","",16],[10,"le","","",16],[10,"gt","","",16],[10,"ge","","",16],[10,"clone","","",16],[10,"eq","","",16],[10,"ne","","",16],[10,"fmt","","",16],[10,"empty","","Returns an empty set of flags.",16],[10,"all","","Returns the set containing all flags.",16],[10,"bits","","Returns the raw value of the flags currently stored.",16],[10,"from_bits","","Convert from underlying bit representation, unless that\nrepresentation contains bits that do not correspond to a flag.",16],[10,"from_bits_truncate","","Convert from underlying bit representation, dropping any bits\nthat do not correspond to flags.",16],[10,"is_empty","","Returns `true` if no flags are currently stored.",16],[10,"is_all","","Returns `true` if all flags are currently set.",16],[10,"intersects","","Returns `true` if there are flags common to both `self` and `other`.",16],[10,"contains","","Returns `true` all of the flags in `other` are contained within `self`.",16],[10,"insert","","Inserts the specified flags in-place.",16],[10,"remove","","Removes the specified flags in-place.",16],[10,"bitor","","Returns the union of the two sets of flags.",16],[10,"bitand","","Returns the intersection between the two sets of flags.",16],[10,"sub","","Returns the set difference of the two sets of flags.",16],[10,"not","","Returns the complement of this set of flags.",16],[0,"Cursor","X11::Window_Attribute",""],[1,"CursorSet","X11::Window_Attribute::Cursor",""],[4,"CursorInt","",""],[5,"none","",""],[10,"hash","","",17],[10,"cmp","","",17],[10,"partial_cmp","","",17],[10,"lt","","",17],[10,"le","","",17],[10,"gt","","",17],[10,"ge","","",17],[10,"clone","","",17],[10,"eq","","",17],[10,"ne","","",17],[10,"fmt","","",17],[10,"empty","","Returns an empty set of flags.",17],[10,"all","","Returns the set containing all flags.",17],[10,"bits","","Returns the raw value of the flags currently stored.",17],[10,"from_bits","","Convert from underlying bit representation, unless that\nrepresentation contains bits that do not correspond to a flag.",17],[10,"from_bits_truncate","","Convert from underlying bit representation, dropping any bits\nthat do not correspond to flags.",17],[10,"is_empty","","Returns `true` if no flags are currently stored.",17],[10,"is_all","","Returns `true` if all flags are currently set.",17],[10,"intersects","","Returns `true` if there are flags common to both `self` and `other`.",17],[10,"contains","","Returns `true` all of the flags in `other` are contained within `self`.",17],[10,"insert","","Inserts the specified flags in-place.",17],[10,"remove","","Removes the specified flags in-place.",17],[10,"bitor","","Returns the union of the two sets of flags.",17],[10,"bitand","","Returns the intersection between the two sets of flags.",17],[10,"sub","","Returns the set difference of the two sets of flags.",17],[10,"not","","Returns the complement of this set of flags.",17],[4,"WindowAttributeInt","X11::Window_Attribute",""],[5,"back_pixmap","",""],[5,"back_pixel","",""],[5,"border_pixmap","",""],[5,"border_pixel","",""],[5,"bit_gravity","",""],[5,"win_gravity","",""],[5,"backing_store","",""],[5,"backing_planes","",""],[5,"backing_pixel","",""],[5,"override_reddirect","",""],[5,"save_under","",""],[5,"event_mask","",""],[5,"dont_propagate","",""],[5,"colormap","",""],[5,"cursor","",""],[10,"hash","","",18],[10,"cmp","","",18],[10,"partial_cmp","","",18],[10,"lt","","",18],[10,"le","","",18],[10,"gt","","",18],[10,"ge","","",18],[10,"clone","","",18],[10,"eq","","",18],[10,"ne","","",18],[10,"fmt","","",18],[10,"empty","","Returns an empty set of flags.",18],[10,"all","","Returns the set containing all flags.",18],[10,"bits","","Returns the raw value of the flags currently stored.",18],[10,"from_bits","","Convert from underlying bit representation, unless that\nrepresentation contains bits that do not correspond to a flag.",18],[10,"from_bits_truncate","","Convert from underlying bit representation, dropping any bits\nthat do not correspond to flags.",18],[10,"is_empty","","Returns `true` if no flags are currently stored.",18],[10,"is_all","","Returns `true` if all flags are currently set.",18],[10,"intersects","","Returns `true` if there are flags common to both `self` and `other`.",18],[10,"contains","","Returns `true` all of the flags in `other` are contained within `self`.",18],[10,"insert","","Inserts the specified flags in-place.",18],[10,"remove","","Removes the specified flags in-place.",18],[10,"bitor","","Returns the union of the two sets of flags.",18],[10,"bitand","","Returns the intersection between the two sets of flags.",18],[10,"sub","","Returns the set difference of the two sets of flags.",18],[10,"not","","Returns the complement of this set of flags.",18],[6,"Cookie","X11","Implementors of this trait represent a pending reply from the X server made via an asynchronous\nrequest."],[9,"wait_for_reply","","",19],[6,"Reply","",""],[10,"fmt","","",20],[10,"screen_setup","","",21],[10,"fmt","","",22],[10,"fmt","","",23],[10,"id","","",23],[10,"root_window","","",22],[10,"drop","","",24],[10,"new","","",25],[10,"force","","Use force to flush all pending requests.\nThe RequestDelay called with the force method is moved into\nthe force method where its destructor is called (once).",25],[10,"subsume","","Use subsume to prevent *other* RequestDelays in the current scope\nfrom calling their destructors (and flushing pending requests).\nThe RequestDelay placed in the “other” parameter\nis moved into the subsume method where its destructor is made to do\nnothing.  No error can occur if subsume is not used, but it helps\ncontrol exactly when pending requests are flushed.",25],[10,"drop","","",25],[10,"new","","Construct a connection to the X server only if it can be shown to be an initially valid\nconnection.\nConnects to the X server specified by the $DISPLAY environment variable (if $DISPLAY\ncan be parsed).",20],[10,"new_with_default_screen","","Does the same as new() but also returns the default Screen if one exists.",20],[10,"status","","Test if connected to the X server and if not return why.",20],[10,"is_valid","","Test if connected to the X server.",20],[10,"flush","","",20],[10,"setup","","See: http://xcb.freedesktop.org/manual/group__XCB__Core__API.html#gafc379a27800bf735818a0760bd036321",20],[10,"make_window_geometry_request","","",20],[10,"make_window_children_request","","",20],[10,"change_window_attributes","","",20],[10,"drop","","Will disconnect the Connection from the X server.",20],[10,"fmt","","",0],[10,"fmt","","",1]],"paths":[[1,"Coordinate"],[1,"RectangularSize"],[2,"ConnectionError"],[2,"ConnectionStatus"],[1,"ScreenSetup"],[1,"Items"],[1,"Cookie"],[1,"WindowGeometry"],[1,"Reply"],[1,"Cookie"],[1,"Reply"],[1,"WindowChildren"],[1,"Items"],[1,"BackPixmapSet"],[1,"BackingStoreSet"],[1,"EventSet"],[1,"ColorMapSet"],[1,"CursorSet"],[1,"WindowAttributeSet"],[6,"Cookie"],[1,"Connection"],[1,"Setup"],[1,"Screen"],[1,"Window"],[1,"RequestError"],[1,"RequestDelay"]]};
initSearch(searchIndex);
