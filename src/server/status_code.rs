use std::mem::swap;

macro_rules! enum_create {
     ($($key: ident => $val: expr),+) => (
            /// `HTTP Response StatusCode`
            #[derive(Debug,Clone,PartialEq,PartialOrd)]
            pub enum StatusCode {
            $($key),+
            }
            impl StatusCode {
                pub fn new<C:Into<u16>>(code:C)->Self {
                    let code= format!("_{}",code.into());
                    match code.as_str() {
                        $(stringify!($key) => StatusCode::$key),+
                        , _=> unreachable!()
                    }
                }
                pub fn set<C:Into<u16>>(&mut self,code:C) {
                    let code= format!("_{}",code.into());
                    match code.as_str() {
                        $(stringify!($key) => *self=StatusCode::$key),+
                        , _=> unreachable!()
                    }
                }
                pub fn swap(&mut self,mut other:&mut StatusCode) {
                    swap(self,&mut other);
                }
                pub fn code(&self)-> u16 {
                    match *self {
                        $(StatusCode::$key => stringify!($key)[1..].parse::<u16>().unwrap()),+                        
                    }
                }
                pub fn desc(&self)->&'static str {
                    match *self {
                        $(StatusCode::$key=>$val),+
                    }
                }
            }
    );
}

impl Default for StatusCode {
    fn default() -> Self {
        StatusCode::new(200_u16)
    }
}

enum_create!(
_0=>"Unknown",
_100=>"Continue",
_101=>"Switching Protocols",
_102=>"Processing",
_118=>"Connection timed out",
_200=>"OK",
_201=>"Created",
_202=>"Accepted",
_203=>"Non-Authoritative Information",
_204=>"No Content",
_205=>"Reset Content",
_206=>"Partial Content",
_207=>"Multi-Status",
_210=>"Content Different",
_300=>"Multiple Choices",
_301=>"Moved Permanently",
_302=>"Found",
_303=>"See Other",
_304=>"Not Modified",
_305=>"Use Proxy",
_307=>"Temporary Redirect",
_400=>"Bad Request",
_401=>"Unauthorized",
_402=>"Payment Required",
_403=>"Forbidden",
_404=>"Not Found",
_405=>"Method Not Allowed",
_406=>"Not Acceptable",
_407=>"Proxy Authentication Required",
_408=>"Request Time-out",
_409=>"Conflict",
_410=>"Gone",
_411=>"Length Required",
_412=>"Precondition Failed",
_413=>"Request Entity Too Large",
_414=>"Reques-URI Too Large",
_415=>"Unsupported Media Type",
_416=>"Request range not satisfiable",
_417=>"Expectation Failed",
_500=>"Internal Server Error",
_501=>"Not Implemented",
_502=>"Bad Gateway",
_503=>"Service Unavailable",
_504=>"Gateway Time-out",
_505=>"HTTP Version not supported"
);
