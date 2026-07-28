//! Raw Win32 API FFI declarations for Findex UI.
//! Zero external dependencies - pure FFI.

#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// ===== Basic Types =====
pub type BOOL = i32;
pub type DWORD = u32;
pub type LONG = i32;
pub type UINT = u32;
pub type WORD = u16;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type LPVOID = *mut std::ffi::c_void;
pub type LPCVOID = *const std::ffi::c_void;
pub type HANDLE = *mut std::ffi::c_void;
pub type HINSTANCE = HANDLE;
pub type HWND = HANDLE;
pub type HMENU = HANDLE;
pub type HICON = HANDLE;
pub type HCURSOR = HANDLE;
pub type HBRUSH = HANDLE;
pub type HFONT = HANDLE;
pub type HBITMAP = HANDLE;
pub type HDC = HANDLE;
pub type HGDIOBJ = HANDLE;
pub type HHOOK = HANDLE;
pub type HACCEL = HANDLE;
pub type HRAWINPUT = HANDLE;
pub type HRGN = HANDLE;
pub type LPARAM_D = isize;
pub type LPCWSTR = *const u16;
pub type LPWSTR = *mut u16;
pub type LPCSTR = *const u8;
pub type LPSTR = *mut u8;
pub type COLORREF = DWORD;
pub type ATOM = WORD;
pub type WPARAM_D = usize;

// ===== Constants =====
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
pub const NULL: usize = 0;

// Window Styles
pub const WS_OVERLAPPED: DWORD = 0x00000000;
pub const WS_POPUP: DWORD = 0x80000000;
pub const WS_CHILD: DWORD = 0x40000000;
pub const WS_MINIMIZE: DWORD = 0x20000000;
pub const WS_VISIBLE: DWORD = 0x10000000;
pub const WS_DISABLED: DWORD = 0x08000000;
pub const WS_CLIPSIBLINGS: DWORD = 0x04000000;
pub const WS_CLIPCHILDREN: DWORD = 0x02000000;
pub const WS_MAXIMIZE: DWORD = 0x01000000;
pub const WS_CAPTION: DWORD = 0x00C00000;
pub const WS_BORDER: DWORD = 0x00800000;
pub const WS_DLGFRAME: DWORD = 0x00400000;
pub const WS_VSCROLL: DWORD = 0x00200000;
pub const WS_HSCROLL: DWORD = 0x00100000;
pub const WS_SYSMENU: DWORD = 0x00080000;
pub const WS_THICKFRAME: DWORD = 0x00040000;
pub const WS_GROUP: DWORD = 0x00020000;
pub const WS_TABSTOP: DWORD = 0x00010000;
pub const WS_MINIMIZEBOX: DWORD = 0x00020000;
pub const WS_MAXIMIZEBOX: DWORD = 0x00010000;
pub const WS_OVERLAPPEDWINDOW: DWORD = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;

// Extended Window Styles
pub const WS_EX_DLGMODALFRAME: DWORD = 0x00000001;
pub const WS_EX_NOPARENTNOTIFY: DWORD = 0x00000004;
pub const WS_EX_TOPMOST: DWORD = 0x00000008;
pub const WS_EX_ACCEPTFILES: DWORD = 0x00000010;
pub const WS_EX_TRANSPARENT: DWORD = 0x00000020;
pub const WS_EX_MDICHILD: DWORD = 0x00000040;
pub const WS_EX_TOOLWINDOW: DWORD = 0x00000080;
pub const WS_EX_WINDOWEDGE: DWORD = 0x00000100;
pub const WS_EX_CLIENTEDGE: DWORD = 0x00000200;
pub const WS_EX_OVERLAPPEDWINDOW: DWORD = WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE;
pub const WS_EX_LAYERED: DWORD = 0x00080000;
pub const WS_EX_NOACTIVATE: DWORD = 0x08000000;
pub const WS_EX_COMPOSITED: DWORD = 0x02000000;

// Show Window commands
pub const SW_HIDE: i32 = 0;
pub const SW_SHOWNORMAL: i32 = 1;
pub const SW_SHOW: i32 = 5;
pub const SW_RESTORE: i32 = 9;
pub const SW_SHOWDEFAULT: i32 = 10;
pub const SW_SHOWMINIMIZED: i32 = 2;

// Window Messages
pub const WM_NULL: UINT = 0x0000;
pub const WM_CREATE: UINT = 0x0001;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_MOVE: UINT = 0x0003;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_ACTIVATE: UINT = 0x0006;
pub const WM_SETFOCUS: UINT = 0x0007;
pub const WM_KILLFOCUS: UINT = 0x0008;
pub const WM_ENABLE: UINT = 0x000A;
pub const WM_SETREDRAW: UINT = 0x000B;
pub const WM_SETTEXT: UINT = 0x000C;
pub const WM_GETTEXT: UINT = 0x000D;
pub const WM_GETTEXTLENGTH: UINT = 0x000E;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_QUIT: UINT = 0x0012;
pub const WM_ERASEBKGND: UINT = 0x0014;
pub const WM_SHOWWINDOW: UINT = 0x0018;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_SYSCOMMAND: UINT = 0x0112;
pub const WM_HOTKEY: UINT = 0x0312;
pub const WM_CTLCOLOREDIT: UINT = 0x0133;
pub const WM_CTLCOLORSTATIC: UINT = 0x0138;
pub const WM_CTLCOLORLISTBOX: UINT = 0x0134;
pub const WM_CTLCOLORBTN: UINT = 0x0135;
pub const WM_CTLCOLORDLG: UINT = 0x0136;
pub const WM_CTLCOLORSCROLLBAR: UINT = 0x0137;
pub const WM_NOTIFY: UINT = 0x004E;
pub const WM_INITDIALOG: UINT = 0x0110;
pub const WM_TIMER: UINT = 0x0113;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_KEYUP: UINT = 0x0101;
pub const WM_SYSKEYDOWN: UINT = 0x0104;
pub const WM_SYSKEYUP: UINT = 0x0105;
pub const WM_CHAR: UINT = 0x0102;
pub const WM_LBUTTONDOWN: UINT = 0x0201;
pub const WM_LBUTTONUP: UINT = 0x0202;
pub const WM_MOUSEMOVE: UINT = 0x0200;
pub const WM_NCLBUTTONDOWN: UINT = 0x00A1;
pub const WM_NCHITTEST: UINT = 0x0084;
pub const WM_GETMINMAXINFO: UINT = 0x0024;
pub const WM_WINDOWPOSCHANGING: UINT = 0x0046;
pub const WM_SETCURSOR: UINT = 0x0020;
pub const WM_ENTERSIZEMOVE: UINT = 0x0231;
pub const WM_EXITSIZEMOVE: UINT = 0x0232;
pub const WM_NCCALCSIZE: UINT = 0x0083;
pub const WM_NCPAINT: UINT = 0x0085;
pub const WM_NCACTIVATE: UINT = 0x0086;
pub const WM_PRINT: UINT = 0x0317;
pub const WM_PRINTCLIENT: UINT = 0x0318;
pub const WM_STYLECHANGED: UINT = 0x007D;
pub const WM_THEMECHANGED: UINT = 0x031A;

// Custom window messages
pub const WM_APP: UINT = 0x8000;

// NOTIFYICONDATA constants
pub const NIM_ADD: UINT = 0;
pub const NIM_MODIFY: UINT = 1;
pub const NIM_DELETE: UINT = 2;
pub const NIF_MESSAGE: UINT = 0x00000001;
pub const NIF_ICON: UINT = 0x00000002;
pub const NIF_TIP: UINT = 0x00000004;

// NOTIFYICONDATA struct
#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: DWORD,
    pub hWnd: HWND,
    pub uID: UINT,
    pub uFlags: UINT,
    pub uCallbackMessage: UINT,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: DWORD,
    pub dwStateMask: DWORD,
    pub szInfo: [u16; 256],
    pub uVersion: UINT,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: DWORD,
    pub guidItem: [u8; 16],
    pub hBalloonIcon: HICON,
}

// Menu constants
pub const MF_STRING: UINT = 0x00000000;
pub const MF_POPUP: UINT = 0x00000010;
pub const MF_SEPARATOR: UINT = 0x00000800;

// TrackPopupMenu flags
pub const TPM_LEFTALIGN: UINT = 0x0000;
pub const TPM_RIGHTBUTTON: UINT = 0x0002;

pub const WM_USER: UINT = 0x0400;

// SYSCOMMAND IDs
pub const SC_CLOSE: UINT = 0xF060;
pub const SC_MINIMIZE: UINT = 0xF020;
pub const SC_MAXIMIZE: UINT = 0xF030;
pub const SC_RESTORE: UINT = 0xF120;

// Standard Cursors
pub const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR;
pub const IDC_IBEAM: LPCWSTR = 32513usize as LPCWSTR;
pub const IDC_WAIT: LPCWSTR = 32514usize as LPCWSTR;
pub const IDC_CROSS: LPCWSTR = 32515usize as LPCWSTR;
pub const IDC_HAND: LPCWSTR = 32649usize as LPCWSTR;

// Standard Icons
pub const IDI_APPLICATION: LPCWSTR = 32512usize as LPCWSTR;
pub const IDI_HAND: LPCWSTR = 32513usize as LPCWSTR;
pub const IDI_QUESTION: LPCWSTR = 32514usize as LPCWSTR;
pub const IDI_EXCLAMATION: LPCWSTR = 32515usize as LPCWSTR;
pub const IDI_ASTERISK: LPCWSTR = 32516usize as LPCWSTR;

// System Colors
pub const COLOR_SCROLLBAR: UINT = 0;
pub const COLOR_BACKGROUND: UINT = 1;
pub const COLOR_ACTIVECAPTION: UINT = 2;
pub const COLOR_INACTIVECAPTION: UINT = 3;
pub const COLOR_MENU: UINT = 4;
pub const COLOR_WINDOW: UINT = 5;
pub const COLOR_WINDOWFRAME: UINT = 6;
pub const COLOR_MENUTEXT: UINT = 7;
pub const COLOR_WINDOWTEXT: UINT = 8;
pub const COLOR_CAPTIONTEXT: UINT = 9;
pub const COLOR_ACTIVEBORDER: UINT = 10;
pub const COLOR_INACTIVEBORDER: UINT = 11;
pub const COLOR_APPWORKSPACE: UINT = 12;
pub const COLOR_HIGHLIGHT: UINT = 13;
pub const COLOR_HIGHLIGHTTEXT: UINT = 14;
pub const COLOR_BTNFACE: UINT = 15;
pub const COLOR_BTNSHADOW: UINT = 16;
pub const COLOR_GRAYTEXT: UINT = 17;
pub const COLOR_BTNTEXT: UINT = 18;
pub const COLOR_INACTIVECAPTIONTEXT: UINT = 19;
pub const COLOR_BTNHIGHLIGHT: UINT = 20;
pub const COLOR_3DDKSHADOW: UINT = 21;
pub const COLOR_3DLIGHT: UINT = 22;
pub const COLOR_INFOTEXT: UINT = 23;
pub const COLOR_INFOBK: UINT = 24;
pub const COLOR_HOTLIGHT: UINT = 26;
pub const COLOR_GRADIENTACTIVECAPTION: UINT = 27;
pub const COLOR_GRADIENTINACTIVECAPTION: UINT = 28;
pub const COLOR_MENUHILIGHT: UINT = 29;
pub const COLOR_MENUBAR: UINT = 30;

// GDI Stock Objects
pub const WHITE_BRUSH: i32 = 0;
pub const LTGRAY_BRUSH: i32 = 1;
pub const GRAY_BRUSH: i32 = 2;
pub const DKGRAY_BRUSH: i32 = 3;
pub const BLACK_BRUSH: i32 = 4;
pub const NULL_BRUSH: i32 = 5;
pub const WHITE_PEN: i32 = 6;
pub const BLACK_PEN: i32 = 7;
pub const NULL_PEN: i32 = 8;
pub const OEM_FIXED_FONT: i32 = 10;
pub const ANSI_FIXED_FONT: i32 = 11;
pub const ANSI_VAR_FONT: i32 = 12;
pub const SYSTEM_FONT: i32 = 13;
pub const DEVICE_DEFAULT_FONT: i32 = 14;
pub const DEFAULT_PALETTE: i32 = 15;
pub const SYSTEM_FIXED_FONT: i32 = 16;
pub const DEFAULT_GUI_FONT: i32 = 17;

// Standard Controls
pub const WC_EDIT: &str = "Edit";
pub const WC_BUTTON: &str = "Button";
pub const WC_STATIC: &str = "Static";
pub const WC_LISTBOX: &str = "ListBox";
pub const WC_COMBOBOX: &str = "ComboBox";
pub const WC_SCROLLBAR: &str = "ScrollBar";
pub const WC_LISTVIEW: &str = "SysListView32";
pub const WC_TREEVIEW: &str = "SysTreeView32";
pub const WC_STATUS: &str = "msctls_statusbar32";
pub const WC_TOOLBAR: &str = "ToolbarWindow32";
pub const WC_PROGRESS: &str = "msctls_progress32";
pub const WC_TRACKBAR: &str = "msctls_trackbar32";
pub const WC_HEADER: &str = "SysHeader32";
pub const WC_TAB: &str = "SysTabControl32";

// Button Styles
pub const BS_PUSHBUTTON: DWORD = 0x00000000;
pub const BS_DEFPUSHBUTTON: DWORD = 0x00000001;
pub const BS_CHECKBOX: DWORD = 0x00000002;
pub const BS_AUTOCHECKBOX: DWORD = 0x00000003;
pub const BS_RADIOBUTTON: DWORD = 0x00000004;
pub const BS_GROUPBOX: DWORD = 0x00000007;
pub const BS_OWNERDRAW: DWORD = 0x0000000B;
pub const BS_LEFT: DWORD = 0x00000100;
pub const BS_RIGHT: DWORD = 0x00000200;
pub const BS_CENTER: DWORD = 0x00000300;
pub const BS_TOP: DWORD = 0x00000400;
pub const BS_BOTTOM: DWORD = 0x00000800;
pub const BS_VCENTER: DWORD = 0x00000C00;
pub const BS_PUSHLIKE: DWORD = 0x00001000;
pub const BS_MULTILINE: DWORD = 0x00002000;
pub const BS_NOTIFY: DWORD = 0x00004000;
pub const BS_FLAT: DWORD = 0x00008000;

// Edit Control Styles
pub const ES_LEFT: DWORD = 0x00000000;
pub const ES_CENTER: DWORD = 0x00000001;
pub const ES_RIGHT: DWORD = 0x00000002;
pub const ES_MULTILINE: DWORD = 0x00000004;
pub const ES_UPPERCASE: DWORD = 0x00000008;
pub const ES_LOWERCASE: DWORD = 0x00000010;
pub const ES_PASSWORD: DWORD = 0x00000020;
pub const ES_AUTOVSCROLL: DWORD = 0x00000040;
pub const ES_AUTOHSCROLL: DWORD = 0x00000080;
pub const ES_NOHIDESEL: DWORD = 0x00000100;
pub const ES_READONLY: DWORD = 0x00000800;
pub const ES_WANTRETURN: DWORD = 0x00001000;

// Static Control Styles
pub const SS_LEFT: DWORD = 0x00000000;
pub const SS_CENTER: DWORD = 0x00000001;
pub const SS_RIGHT: DWORD = 0x00000002;
pub const SS_ICON: DWORD = 0x00000003;
pub const SS_BLACKRECT: DWORD = 0x00000004;
pub const SS_GRAYRECT: DWORD = 0x00000005;
pub const SS_WHITERECT: DWORD = 0x00000006;
pub const SS_BLACKFRAME: DWORD = 0x00000007;
pub const SS_GRAYFRAME: DWORD = 0x00000008;
pub const SS_WHITEFRAME: DWORD = 0x00000009;
pub const SS_SIMPLE: DWORD = 0x0000000B;
pub const SS_LEFTNOWORDWRAP: DWORD = 0x0000000C;
pub const SS_OWNERDRAW: DWORD = 0x0000000D;
pub const SS_BITMAP: DWORD = 0x0000000E;
pub const SS_ENHMETAFILE: DWORD = 0x0000000F;
pub const SS_ETCHEDHORZ: DWORD = 0x00000010;
pub const SS_ETCHEDVERT: DWORD = 0x00000011;
pub const SS_ETCHEDFRAME: DWORD = 0x00000012;
pub const SS_TYPEMASK: DWORD = 0x0000001F;
pub const SS_REALSIZECONTROL: DWORD = 0x00000040;
pub const SS_NOPREFIX: DWORD = 0x00000080;
pub const SS_NOTIFY: DWORD = 0x00000100;
pub const SS_CENTERIMAGE: DWORD = 0x00000200;
pub const SS_RIGHTJUST: DWORD = 0x00000400;
pub const SS_REALSIZEIMAGE: DWORD = 0x00000800;
pub const SS_SUNKEN: DWORD = 0x00001000;
pub const SS_EDITCONTROL: DWORD = 0x00002000;
pub const SS_ENDELLIPSIS: DWORD = 0x00004000;
pub const SS_PATHELLIPSIS: DWORD = 0x00008000;
pub const SS_WORDELLIPSIS: DWORD = 0x0000C000;
pub const SS_ELLIPSISMASK: DWORD = 0x0000C000;

// ListBox Styles
pub const LBS_NOTIFY: DWORD = 0x00000001;
pub const LBS_SORT: DWORD = 0x00000002;
pub const LBS_NOREDRAW: DWORD = 0x00000004;
pub const LBS_MULTIPLESEL: DWORD = 0x00000008;
pub const LBS_OWNERDRAWFIXED: DWORD = 0x00000010;
pub const LBS_OWNERDRAWVARIABLE: DWORD = 0x00000020;
pub const LBS_HASSTRINGS: DWORD = 0x00000040;
pub const LBS_USETABSTOPS: DWORD = 0x00000080;
pub const LBS_NOINTEGRALHEIGHT: DWORD = 0x00000100;
pub const LBS_MULTICOLUMN: DWORD = 0x00000200;
pub const LBS_WANTKEYBOARDINPUT: DWORD = 0x00000400;
pub const LBS_EXTENDEDSEL: DWORD = 0x00000800;
pub const LBS_DISABLENOSCROLL: DWORD = 0x00001000;
pub const LBS_NODATA: DWORD = 0x00002000;
pub const LBS_STANDARD: DWORD = LBS_NOTIFY | LBS_SORT | WS_VSCROLL | WS_BORDER;

// Button messages
pub const BM_GETCHECK: UINT = 0x00F0;
pub const BM_SETCHECK: UINT = 0x00F1;

// ListBox Messages
pub const LB_ADDSTRING: UINT = 0x0180;
pub const LB_INSERTSTRING: UINT = 0x0181;
pub const LB_DELETESTRING: UINT = 0x0182;
pub const LB_SELCHANGE: UINT = 0x0183;
pub const LB_GETSELCOUNT: UINT = 0x0190;
pub const LB_GETSELITEMS: UINT = 0x0191;
pub const LB_GETTEXT: UINT = 0x0189;
pub const LB_GETTEXTLEN: UINT = 0x018A;
pub const LB_GETCOUNT: UINT = 0x018B;
pub const LB_GETCURSEL: UINT = 0x0188;
pub const LB_SETCURSEL: UINT = 0x0186;
pub const LB_RESETCONTENT: UINT = 0x0184;
pub const LB_FINDSTRING: UINT = 0x018F;
pub const LB_ADDSTRING_S: UINT = LB_ADDSTRING;

// Edit Control Messages
pub const EM_GETSEL: UINT = 0x00B0;
pub const EM_SETSEL: UINT = 0x00B1;
pub const EM_GETRECT: UINT = 0x00B2;
pub const EM_SETRECT: UINT = 0x00B3;
pub const EM_SETRECTNP: UINT = 0x00B4;
pub const EM_SCROLL: UINT = 0x00B5;
pub const EM_LINESCROLL: UINT = 0x00B6;
pub const EM_SCROLLCARET: UINT = 0x00B7;
pub const EM_GETLIMITTEXT: UINT = 0x00D5;
pub const EM_SETLIMITTEXT: UINT = 0x00C5;
pub const EM_REPLACESEL: UINT = 0x00C2;
pub const EM_GETLINE: UINT = 0x00C4;
pub const EM_LINELENGTH: UINT = 0x00C1;
pub const EM_GETLINECOUNT: UINT = 0x00BA;
pub const EM_SETMODIFY: UINT = 0x00B9;
pub const EM_GETMODIFY: UINT = 0x00B8;

// Command notification codes
pub const BN_CLICKED: UINT = 0;
pub const EN_CHANGE: UINT = 0x0300;
pub const EN_UPDATE: UINT = 0x0400;
pub const EN_SETFOCUS: UINT = 0x0100;
pub const EN_KILLFOCUS: UINT = 0x0200;
pub const LBN_SELCHANGE: UINT = 1;
pub const LBN_DBLCLK: UINT = 2;

// GDI Text alignment
pub const DT_LEFT: UINT = 0x00000000;
pub const DT_CENTER: UINT = 0x00000001;
pub const DT_RIGHT: UINT = 0x00000002;
pub const DT_VCENTER: UINT = 0x00000004;
pub const DT_TOP: UINT = 0x00000000;
pub const DT_BOTTOM: UINT = 0x00000008;
pub const DT_WORDBREAK: UINT = 0x00000010;
pub const DT_SINGLELINE: UINT = 0x00000020;
pub const DT_EXPANDTABS: UINT = 0x00000040;
pub const DT_TABSTOP: UINT = 0x00000080;
pub const DT_NOCLIP: UINT = 0x00000100;
pub const DT_EXTERNALLEADING: UINT = 0x00000200;
pub const DT_CALCRECT: UINT = 0x00000400;
pub const DT_NOPREFIX: UINT = 0x00000800;
pub const DT_INTERNAL: UINT = 0x00001000;
pub const DT_EDITCONTROL: UINT = 0x00002000;
pub const DT_PATH_ELLIPSIS: UINT = 0x00004000;
pub const DT_END_ELLIPSIS: UINT = 0x00008000;
pub const DT_MODIFYSTRING: UINT = 0x00010000;
pub const DT_RTLREADING: UINT = 0x00020000;
pub const DT_WORD_ELLIPSIS: UINT = 0x00040000;
pub const DT_NOFULLWIDTHCHARBREAK: UINT = 0x00080000;
pub const DT_HIDEPREFIX: UINT = 0x00100000;
pub const DT_PREFIXONLY: UINT = 0x00200000;

// GDI Raster Operations
pub const SRCCOPY: DWORD = 0x00CC0020;
pub const SRCPAINT: DWORD = 0x00EE0086;
pub const SRCAND: DWORD = 0x008800C6;
pub const SRCINVERT: DWORD = 0x00660046;
pub const SRCERASE: DWORD = 0x00440328;
pub const NOTSRCCOPY: DWORD = 0x00330008;
pub const NOTSRCERASE: DWORD = 0x001100A6;
pub const MERGECOPY: DWORD = 0x00C000CA;
pub const MERGEPAINT: DWORD = 0x00BB0226;
pub const PATCOPY: DWORD = 0x00F00021;
pub const PATPAINT: DWORD = 0x00FB0A09;
pub const PATINVERT: DWORD = 0x005A0049;
pub const DSTINVERT: DWORD = 0x00550009;
pub const BLACKNESS: DWORD = 0x00000042;
pub const WHITENESS: DWORD = 0x00FF0062;

// GDI Pen Styles
pub const PS_SOLID: UINT = 0;
pub const PS_DASH: UINT = 1;
pub const PS_DOT: UINT = 2;
pub const PS_DASHDOT: UINT = 3;
pub const PS_DASHDOTDOT: UINT = 4;
pub const PS_NULL: UINT = 5;
pub const PS_INSIDEFRAME: UINT = 6;

// GDI Brush Styles
pub const BS_SOLID: UINT = 0;
pub const BS_NULL: UINT = 1;
pub const BS_HOLLOW: UINT = 1;
pub const BS_HATCHED: UINT = 2;
pub const BS_PATTERN: UINT = 3;
pub const BS_INDEXED: UINT = 4;
pub const BS_DIBPATTERN: UINT = 5;
pub const BS_DIBPATTERNPT: UINT = 6;
pub const BS_PATTERN8X8: UINT = 7;
pub const BS_MONOPATTERN: UINT = 9;

// UpdateLayeredWindow constants
pub const ULW_COLORKEY: DWORD = 0x00000001;
pub const ULW_ALPHA: DWORD = 0x00000002;
pub const ULW_OPAQUE: DWORD = 0x00000004;

// Layered window attributes
pub const LWA_COLORKEY: DWORD = 0x00000001;
pub const LWA_ALPHA: DWORD = 0x00000002;

// SetWindowPos flags
pub const SWP_NOSIZE: UINT = 0x0001;
pub const SWP_NOMOVE: UINT = 0x0002;
pub const SWP_NOZORDER: UINT = 0x0004;
pub const SWP_NOREDRAW: UINT = 0x0008;
pub const SWP_NOACTIVATE: UINT = 0x0010;
pub const SWP_FRAMECHANGED: UINT = 0x0020;
pub const SWP_SHOWWINDOW: UINT = 0x0040;
pub const SWP_HIDEWINDOW: UINT = 0x0080;
pub const SWP_NOCOPYBITS: UINT = 0x0100;
pub const SWP_NOOWNERZORDER: UINT = 0x0200;
pub const SWP_NOSENDCHANGING: UINT = 0x0400;
pub const SWP_DRAWFRAME: UINT = SWP_FRAMECHANGED;
pub const SWP_NOREPOSITION: UINT = SWP_NOOWNERZORDER;
pub const SWP_DEFERERASE: UINT = 0x2000;
pub const SWP_ASYNCWINDOWPOS: UINT = 0x4000;

// Windows Hook IDs
pub const WH_KEYBOARD_LL: i32 = 13;
pub const WH_MOUSE_LL: i32 = 14;
pub const WH_KEYBOARD: i32 = 2;
pub const WH_MOUSE: i32 = 7;

// Virtual Key Codes
pub const VK_CONTROL: i32 = 0x11;
pub const VK_MENU: i32 = 0x12; // Alt
pub const VK_SHIFT: i32 = 0x10;
pub const VK_SPACE: i32 = 0x20;
pub const VK_RETURN: i32 = 0x0D;
pub const VK_ESCAPE: i32 = 0x1B;
pub const VK_TAB: i32 = 0x09;
pub const VK_BACK: i32 = 0x08;
pub const VK_DELETE: i32 = 0x2E;
pub const VK_UP: i32 = 0x26;
pub const VK_DOWN: i32 = 0x28;
pub const VK_LEFT: i32 = 0x25;
pub const VK_RIGHT: i32 = 0x27;
pub const VK_HOME: i32 = 0x24;
pub const VK_END: i32 = 0x23;
pub const VK_PRIOR: i32 = 0x21; // Page Up
pub const VK_NEXT: i32 = 0x22; // Page Down
pub const VK_F1: i32 = 0x70;
pub const VK_F2: i32 = 0x71;
pub const VK_F3: i32 = 0x72;
pub const VK_F4: i32 = 0x73;
pub const VK_F5: i32 = 0x74;
pub const VK_F6: i32 = 0x75;
pub const VK_F7: i32 = 0x76;
pub const VK_F8: i32 = 0x77;
pub const VK_F9: i32 = 0x78;
pub const VK_F10: i32 = 0x79;
pub const VK_F11: i32 = 0x7A;
pub const VK_F12: i32 = 0x7B;
pub const VK_OEM_1: i32 = 0xBA; // ';:'
pub const VK_OEM_PLUS: i32 = 0xBB; // '=+'
pub const VK_OEM_COMMA: i32 = 0xBC; // ',<'
pub const VK_OEM_MINUS: i32 = 0xBD; // '-_'
pub const VK_OEM_PERIOD: i32 = 0xBE; // '.>'
pub const VK_OEM_2: i32 = 0xBF; // '/?'
pub const VK_OEM_3: i32 = 0xC0; // '`~'
pub const VK_OEM_4: i32 = 0xDB; // '[{'
pub const VK_OEM_5: i32 = 0xDC; // '\|'
pub const VK_OEM_6: i32 = 0xDD; // ']}'
pub const VK_OEM_7: i32 = 0xDE; // ''"'

// Modifier keys for RegisterHotKey
pub const MOD_ALT: DWORD = 0x0001;
pub const MOD_CONTROL: DWORD = 0x0002;
pub const MOD_NOREPEAT: DWORD = 0x4000;
pub const MOD_SHIFT: DWORD = 0x0004;
pub const MOD_WIN: DWORD = 0x0008;

// PeekMessage/GetMessage flags
pub const PM_NOREMOVE: UINT = 0x0000;
pub const PM_REMOVE: UINT = 0x0001;
pub const PM_NOYIELD: UINT = 0x0002;
pub const PM_QS_INPUT: UINT = 0x07000000;
pub const PM_QS_PAINT: UINT = 0x00200000;
pub const PM_QS_POSTMESSAGE: UINT = 0x00800000;
pub const PM_QS_SENDMESSAGE: UINT = 0x00400000;

// Input types for SendInput
pub const INPUT_MOUSE: DWORD = 0;
pub const INPUT_KEYBOARD: DWORD = 1;
pub const INPUT_HARDWARE: DWORD = 2;

// KEYBDINPUT flags
pub const KEYEVENTF_EXTENDEDKEY: DWORD = 0x0001;
pub const KEYEVENTF_KEYUP: DWORD = 0x0002;
pub const KEYEVENTF_SCANCODE: DWORD = 0x0008;
pub const KEYEVENTF_UNICODE: DWORD = 0x0004;

// DLL loading
pub const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: DWORD = 0x00001000;

// ===== Structures =====

#[repr(C)]
#[derive(Clone)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

#[repr(C)]
#[derive(Clone)]
pub struct SIZE {
    pub cx: LONG,
    pub cy: LONG,
}

#[repr(C)]
#[derive(Clone)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}

impl RECT {
    pub fn width(&self) -> LONG { self.right - self.left }
    pub fn height(&self) -> LONG { self.bottom - self.top }
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
    pub hIconSm: HICON,
}

#[repr(C)]
#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [u8; 32],
}

// Bitmap info
#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub biSize: DWORD,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: WORD,
    pub biBitCount: WORD,
    pub biCompression: DWORD,
    pub biSizeImage: DWORD,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: DWORD,
    pub biClrImportant: DWORD,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [DWORD; 1],
}

pub const BI_RGB: DWORD = 0;
pub const DIB_RGB_COLORS: UINT = 0;

#[repr(C)]
pub struct CREATESTRUCTW {
    pub lpCreateParams: LPVOID,
    pub hInstance: HINSTANCE,
    pub hMenu: HMENU,
    pub hwndParent: HWND,
    pub cy: i32,
    pub cx: i32,
    pub y: i32,
    pub x: i32,
    pub style: LONG,
    pub lpszName: LPCWSTR,
    pub lpszClass: LPCWSTR,
    pub dwExStyle: DWORD,
}

#[repr(C)]
pub struct MINMAXINFO {
    pub ptReserved: POINT,
    pub ptMaxSize: POINT,
    pub ptMaxPosition: POINT,
    pub ptMinTrackSize: POINT,
    pub ptMaxTrackSize: POINT,
}

#[repr(C)]
pub struct WINDOWPOS {
    pub hwnd: HWND,
    pub hwndInsertAfter: HWND,
    pub x: i32,
    pub y: i32,
    pub cx: i32,
    pub cy: i32,
    pub flags: UINT,
}

#[repr(C)]
pub struct NCCALCSIZE_PARAMS {
    pub rgrc: [RECT; 3],
    pub lppos: *mut WINDOWPOS,
}

#[repr(C)]
pub struct LOGBRUSH {
    pub lbStyle: UINT,
    pub lbColor: COLORREF,
    pub lbHatch: usize,
}

#[repr(C)]
pub struct LOGFONTW {
    pub lfHeight: LONG,
    pub lfWidth: LONG,
    pub lfEscapement: LONG,
    pub lfOrientation: LONG,
    pub lfWeight: LONG,
    pub lfItalic: u8,
    pub lfUnderline: u8,
    pub lfStrikeOut: u8,
    pub lfCharSet: u8,
    pub lfOutPrecision: u8,
    pub lfClipPrecision: u8,
    pub lfQuality: u8,
    pub lfPitchAndFamily: u8,
    pub lfFaceName: [u16; 32],
}

#[repr(C)]
pub struct KBDLLHOOKSTRUCT {
    pub vkCode: DWORD,
    pub scanCode: DWORD,
    pub flags: DWORD,
    pub time: DWORD,
    pub dwExtraInfo: usize,
}

#[repr(C)]
pub struct MSLLHOOKSTRUCT {
    pub pt: POINT,
    pub mouseData: DWORD,
    pub flags: DWORD,
    pub time: DWORD,
    pub dwExtraInfo: usize,
}

// ===== FFI Functions =====

#[link(name = "gdi32")]
#[link(name = "comctl32")]
#[link(name = "shell32")]
#[link(name = "comdlg32")]
extern "system" {
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HINSTANCE;
    pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM;
    pub fn UnregisterClassW(lpClassName: LPCWSTR, hInstance: HINSTANCE) -> BOOL;
    pub fn CreateWindowExW(
        dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR,
        dwStyle: DWORD, X: i32, Y: i32, nWidth: i32, nHeight: i32,
        hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID,
    ) -> HWND;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL;
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: i32) -> HWND;
    pub fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: i32) -> i32;
    pub fn GetWindowTextLengthW(hWnd: HWND) -> i32;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    pub fn IsWindowVisible(hWnd: HWND) -> BOOL;
    pub fn IsWindow(hWnd: HWND) -> BOOL;
    pub fn MoveWindow(hWnd: HWND, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: BOOL) -> BOOL;
    pub fn SetWindowPos(hWnd: HWND, hWndInsertAfter: HWND, X: i32, Y: i32, cx: i32, cy: i32, uFlags: UINT) -> BOOL;
    pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn ClientToScreen(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
    pub fn ScreenToClient(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
    pub fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    pub fn GetForegroundWindow() -> HWND;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn GetFocus() -> HWND;
    pub fn GetDesktopWindow() -> HWND;
    pub fn GetParent(hWnd: HWND) -> HWND;
    pub fn SetParent(hWndChild: HWND, hWndNewParent: HWND) -> HWND;
    pub fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND;
    pub fn FindWindowExW(hWndParent: HWND, hWndChildAfter: HWND, lpszClass: LPCWSTR, lpszWindow: LPCWSTR) -> HWND;
    pub fn EnumWindows(lpEnumFunc: Option<unsafe extern "system" fn(HWND, LPARAM) -> BOOL>, lParam: LPARAM) -> BOOL;
    pub fn GetClassNameW(hWnd: HWND, lpClassName: LPWSTR, nMaxCount: i32) -> i32;
    pub fn BringWindowToTop(hWnd: HWND) -> BOOL;
    pub fn SetWindowLongW(hWnd: HWND, nIndex: i32, dwNewLong: LONG) -> LONG;
    pub fn GetWindowLongW(hWnd: HWND, nIndex: i32) -> LONG;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
    pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
    pub fn RedrawWindow(hWnd: HWND, lprcUpdate: *const RECT, hrgnUpdate: HANDLE, flags: UINT) -> BOOL;
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    pub fn PeekMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, wRemoveMsg: UINT) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn DefWindowProcW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn DefDlgProcW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SendDlgItemMessageW(hWnd: HWND, nIDDlgItem: i32, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON;
    pub fn SetCursor(hCursor: HCURSOR) -> HCURSOR;
    pub fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    pub fn SetCursorPos(x: i32, y: i32) -> BOOL;
    pub fn GetSystemMetrics(nIndex: i32) -> i32;
    pub fn GetDC(hWnd: HWND) -> HDC;
    pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
    pub fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    pub fn GetStockObject(i: i32) -> HGDIOBJ;
    pub fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    pub fn CreatePen(fnStyle: i32, nWidth: i32, crColor: COLORREF) -> HGDIOBJ;
    pub fn CreateFontW(nHeight: i32, nWidth: i32, nEscapement: i32, nOrientation: i32, fnWeight: i32, fdwItalic: DWORD, fdwUnderline: DWORD, fdwStrikeOut: DWORD, fdwCharSet: DWORD, fdwOutputPrecision: DWORD, fdwClipPrecision: DWORD, fdwQuality: DWORD, fdwPitchAndFamily: DWORD, lpszFace: LPCWSTR) -> HFONT;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn Rectangle(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32) -> BOOL;
    pub fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    pub fn FrameRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    pub fn DrawTextW(hdc: HDC, lpchText: LPCWSTR, cchText: i32, lprc: *mut RECT, format: UINT) -> i32;
    pub fn TextOutW(hdc: HDC, x: i32, y: i32, lpString: LPCWSTR, c: i32) -> BOOL;
    pub fn GetTextExtentPoint32W(hdc: HDC, lpString: LPCWSTR, c: i32, psizl: *mut SIZE) -> BOOL;
    pub fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn GetPixel(hdc: HDC, x: i32, y: i32) -> COLORREF;
    pub fn MoveToEx(hdc: HDC, x: i32, y: i32, lppt: *mut POINT) -> BOOL;
    pub fn LineTo(hdc: HDC, x: i32, y: i32) -> BOOL;
    pub fn SetViewportOrgEx(hdc: HDC, x: i32, y: i32, lppt: *mut POINT) -> BOOL;
    pub fn GetViewportOrgEx(hdc: HDC, lppt: *mut POINT) -> BOOL;
    pub fn BitBlt(hdc: HDC, x: i32, y: i32, cx: i32, cy: i32, hdcSrc: HDC, x1: i32, y1: i32, rop: DWORD) -> BOOL;
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP;
    pub fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: DWORD, vk: UINT) -> BOOL;
    pub fn UnregisterHotKey(hWnd: HWND, id: i32) -> BOOL;
    pub fn SetWindowsHookExW(idHook: i32, lpfn: HOOKPROC, hmod: HINSTANCE, dwThreadId: DWORD) -> HHOOK;
    pub fn UnhookWindowsHookEx(hhk: HHOOK) -> BOOL;
    pub fn CallNextHookEx(hhk: HHOOK, nCode: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn GetLastError() -> DWORD;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn wsprintfW(output: LPWSTR, format: LPCWSTR, ...) -> i32;
    pub fn lstrlenW(lpString: LPCWSTR) -> i32;
    pub fn MessageBoxW(hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT) -> i32;
    pub fn GetKeyState(nVirtKey: i32) -> i16;
    pub fn GetAsyncKeyState(vKey: i32) -> i16;
    pub fn WaitMessage() -> BOOL;
    pub fn MsgWaitForMultipleObjects(nCount: DWORD, pHandles: *const HANDLE, fWaitAll: BOOL, dwMilliseconds: DWORD, dwWakeMask: DWORD) -> DWORD;
    pub fn GetTickCount() -> DWORD;
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn GetCurrentThreadId() -> DWORD;
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn OutputDebugStringW(lpOutputString: LPCWSTR);
    pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HINSTANCE;
    pub fn FreeLibrary(hLibModule: HINSTANCE) -> BOOL;
    pub fn GetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> LPVOID;
    pub fn SetTimer(hWnd: HWND, nIDEvent: usize, uElapse: UINT, lpTimerFunc: Option<unsafe extern "system" fn(HWND, UINT, usize, DWORD)>) -> usize;
    pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> BOOL;
    pub fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
    pub fn CloseClipboard() -> BOOL;
    pub fn EmptyClipboard() -> BOOL;
    pub fn SetClipboardData(uFormat: UINT, hMem: HANDLE) -> HANDLE;
    pub fn GetClipboardData(uFormat: UINT) -> HANDLE;
    pub fn IsClipboardFormatAvailable(uFormat: UINT) -> BOOL;
    pub fn GlobalAlloc(uFlags: UINT, dwBytes: usize) -> HANDLE;
    pub fn GlobalLock(hMem: HANDLE) -> LPVOID;
    pub fn GlobalUnlock(hMem: HANDLE) -> BOOL;
    pub fn GlobalFree(hMem: HANDLE) -> HANDLE;
    pub fn ShellExecuteW(hwnd: HWND, lpOperation: LPCWSTR, lpFile: LPCWSTR, lpParameters: LPCWSTR, lpDirectory: LPCWSTR, nShowCmd: i32) -> HINSTANCE;
    pub fn StretchDIBits(hdc: HDC, xDest: i32, yDest: i32, wDest: i32, hDest: i32, xSrc: i32, ySrc: i32, wSrc: i32, hSrc: i32, lpBits: LPCVOID, lpbmi: *const BITMAPINFO, iUsage: UINT, dwRop: DWORD) -> i32;
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD;

    pub fn SHBrowseForFolderW(lpbi: *mut BROWSEINFOW) -> LPVOID;
    pub fn SHGetPathFromIDListW(pidl: LPVOID, pszPath: LPWSTR) -> BOOL;
    pub fn Shell_NotifyIconW(dwMessage: DWORD, lpdata: *mut NOTIFYICONDATAW) -> BOOL;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(hMenu: HMENU, uFlags: UINT, uIDNewItem: usize, lpNewItem: LPCWSTR) -> BOOL;
    pub fn TrackPopupMenu(hMenu: HMENU, uFlags: UINT, x: i32, y: i32, nReserved: i32, hWnd: HWND, prcRect: LPVOID) -> BOOL;
    pub fn DestroyMenu(hMenu: HMENU) -> BOOL;
    pub fn GetMenuItemCount(hMenu: HMENU) -> i32;    pub fn CoTaskMemFree(pv: LPVOID);
    pub fn CallWindowProcW(lpPrevWndFunc: LPVOID, hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SetWindowRgn(hWnd: HWND, hRgn: HRGN, bRedraw: BOOL) -> i32;
    pub fn CreateRoundRectRgn(x1: i32, y1: i32, x2: i32, y2: i32, w: i32, h: i32) -> HRGN;
}

pub type HOOKPROC = unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT;

// ===== Helper Functions =====

pub fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

pub fn from_wstring(ptr: LPCWSTR) -> String {
    if ptr.is_null() { return String::new(); }
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 { len += 1; }
        if len == 0 { return String::new(); }
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16_lossy(slice)
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    (r as DWORD) | ((g as DWORD) << 8) | ((b as DWORD) << 16)
}

pub fn get_r(rgb: COLORREF) -> u8 { rgb as u8 }
pub fn get_g(rgb: COLORREF) -> u8 { (rgb >> 8) as u8 }
pub fn get_b(rgb: COLORREF) -> u8 { (rgb >> 16) as u8 }

pub fn hiword(l: DWORD) -> WORD { (l >> 16) as WORD }
pub fn loword(l: DWORD) -> WORD { l as WORD }

pub const WM_SETFONT: UINT = 0x0030;


// Browse for folder
#[repr(C)]
pub struct BROWSEINFOW {
    pub hwndOwner: HWND,
    pub pidlRoot: LPVOID,
    pub pszDisplayName: LPWSTR,
    pub lpszTitle: LPCWSTR,
    pub ulFlags: UINT,
    pub lpfn: LPVOID,
    pub lParam: LPARAM,
    pub iImage: i32,
}

// BIF flags
pub const BIF_RETURNONLYFSDIRS: UINT = 0x0001;
pub const BIF_DONTGOBELOWDOMAIN: UINT = 0x0002;
pub const BIF_NEWDIALOGSTYLE: UINT = 0x0040;
pub const BIF_EDITBOX: UINT = 0x0010;
pub const BIF_USENEWUI: UINT = BIF_NEWDIALOGSTYLE | BIF_EDITBOX;

// GWLP indices for SetWindowLongPtrW / GetWindowLongPtrW
pub const GWLP_WNDPROC: i32 = -4;
pub const GWLP_USERDATA: i32 = -21;



