// https://github.com/svartalf/hostname/blob/master/src/nix.rs

use std::ffi::CStr;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::os::unix::ffi::OsStrExt;
// use std::os::unix::process::CommandExt;
use std::sync::Arc;
use std::sync::RwLock;

use libc::sysctlbyname;
use objc::{class, msg_send};

use crate::macos::util::id;

use super::preview_all_classes;
const _POSIX_HOST_NAME_MAX: libc::c_long = 255;

pub struct NickName(pub(crate) Arc<RwLock<id>>);

impl Debug for NickName {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("NickName").finish()
    }
}

// https://github.com/servo/core-foundation-rs/blob/d4ce710182f1756c9d874ab917283fe1a1b7a011/cocoa/src/appkit.rs#L3890
#[link(name = "AppKit", kind = "framework")]
extern "C" {
    // System image names (NSString const*)
    pub static NSImageNameComputer: *mut objc::runtime::AnyObject;
}

impl NickName {
    pub fn new() -> crate::Result<Self> {
        // preview_all_classes();

        Ok(Self(Arc::new(RwLock::new(unsafe {
            // // Create an instance of the UIDevice class
            // let superclass = class!(NSObject);
            // // デバッグ用にsuperclassの情報を出力する
            // println!("superclass: {:?}", superclass);

            // // let panel = class!(NSWindow);
            // // println!("panel: {:?}", panel);

            // objc::runtime::AnyClass::get("NSObject").unwrap();

            // let mut cmd = std::process::Command::new("scutil");
            // cmd.arg("--get").arg("ComputerName");
            // let out = cmd.output().unwrap().stdout;
            // let out = String::from_utf8(out).unwrap();
            // // out: Mac-1702909720453.local
            // println!("out: {}", out);

            msg_send![class!(NSObject), alloc]
        }))))
    }

    pub fn get(&self) -> crate::Result<String> {
        // let hostname = self.get_by_gethostname()?;

        // let name = self.get_by_sysctlbyname()?;
        // println!("name: {}", name);
        // name: Mac-1702909720453.local

        let hostname = "default".into();

        Ok(hostname)
    }

    pub fn get_by_gethostname(&self) -> crate::Result<String> {
        // ホスト名を格納するバッファのサイズを指定
        // https://pubs.opengroup.org/onlinepubs/9699919799/functions/gethostname.html
        let limit = unsafe { libc::sysconf(libc::_SC_HOST_NAME_MAX) };
        let size = libc::c_long::max(limit, _POSIX_HOST_NAME_MAX) as usize;
        let mut hostname_buffer: Vec<u8> = vec![0; size + 1];

        // libcのgethostname関数を呼び出し、ホスト名を取得
        let result =
            unsafe { libc::gethostname(hostname_buffer.as_mut_ptr() as *mut libc::c_char, size) };

        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        // ヌル終端されたC文字列をRustの文字列に変換
        let hostname_cstr =
            unsafe { CStr::from_ptr(hostname_buffer.as_ptr() as *const libc::c_char) };
        match hostname_cstr.to_str() {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(crate::Error::Other(Box::new(e))),
        }
    }

    pub fn get_by_sysctlbyname(&self) -> crate::Result<String> {
        let mut mib: [libc::c_int; 2] = [0, 0];
        let mut len: libc::size_t = 0;

        mib[0] = libc::CTL_KERN;
        mib[1] = libc::KERN_HOSTNAME;

        let result = unsafe {
            sysctlbyname(
                "kern.hostname\0".as_ptr() as *const libc::c_char,
                std::ptr::null_mut(),
                &mut len,
                std::ptr::null_mut(),
                0,
            )
        };

        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        len += 1;

        let mut hostname_buffer: Vec<u8> = vec![0; len];

        let result = unsafe {
            sysctlbyname(
              "kern.hostname\0".as_ptr() as *const libc::c_char,
              hostname_buffer.as_mut_ptr() as *mut libc::c_void,
              &mut len,
              std::ptr::null_mut(),
              0,
            )
          };

        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        let hostname_cstr =
            unsafe { CStr::from_ptr(hostname_buffer.as_ptr() as *const libc::c_char) };
        let hostname = hostname_cstr.to_str().unwrap();

        Ok(hostname.into())
    }

    #[allow(non_snake_case)]
    /// Darling return "NSComputer"
    pub fn get_by_NSImageNameComputer(&self) -> crate::Result<String> {
        if unsafe { NSImageNameComputer }.is_null() {
            Err(crate::Error::OsNotSupported(
                "NSImageNameComputer is null".into(),
            ))
        } else {
            // https://github.com/servo/core-foundation-rs/blob/d4ce710182f1756c9d874ab917283fe1a1b7a011/cocoa-foundation/src/foundation.rs#L632
            let rust_string: *const libc::c_char =
                unsafe { msg_send![NSImageNameComputer, UTF8String] };
            let rust_string = unsafe { CStr::from_ptr(rust_string) };
            let rust_string = rust_string.to_str().unwrap();
            Ok(rust_string.into())
        }
    }

    pub fn set<S: Into<String>>(&self, nickname: S) -> crate::Result<()> {
        let nickname: String = nickname.into();
        self.set_hostname(&nickname)?;

        let name = self.get()?;
        println!("set name: {}", name);

        let mut cmd = std::process::Command::new("scutil");
        cmd.arg("--set").arg("ComputerName").arg(nickname);
        let out = cmd.output().unwrap().stdout;
        let out = String::from_utf8(out).unwrap();
        // out: Mac-1702909720453.local
        println!("scutil set out: {}", out);

        let name = self.get()?;
        println!("set name: {}", name);

        Ok(())
    }

    // https://github.com/svartalf/hostname/blob/master/src/nix.rs
    pub fn set_hostname<S: Into<String>>(&self, nickname: S) -> crate::Result<()> {
        let nickname: String = nickname.into();
        let nickname = std::ffi::OsStr::new(&nickname);

        if nickname.len() > libc::c_int::MAX as usize {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "nickname too long").into());
        }

        let size = nickname.len() as libc::c_int;

        let result =
            unsafe { libc::sethostname(nickname.as_bytes().as_ptr() as *const libc::c_char, size) };

        if result != 0 {
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(())
        }
    }
}