use crate::error::{Result, anyhow, throw};
use jni::sys::jlong;
use replace_with::replace_with_or_abort;

pub type Point = usize;

pub trait ToJavaPoint {
    fn to_point(self) -> jlong;
}

impl ToJavaPoint for Point {
    fn to_point(self) -> jlong {
        self as jlong
    }
}

#[inline]
pub fn to_ptr<T>(s: T) -> Point {
    Box::into_raw(Box::new(s)) as Point
}

#[inline]
fn check_ptr<T>(point: *mut T) -> Result<()> {
    if point.is_null() {
        return throw("point is null or not");
    }
    Ok(())
}

#[inline]
pub fn to_status_use<T>(p: Point) -> Result<&'static mut T> {
    let point = p as *mut T;
    check_ptr(point)?;
    unsafe {
        point
            .as_mut()
            .ok_or_else(|| anyhow!("read pointer error: ({})", p))
    }
}

#[inline]
pub fn to_status_replace<T>(p: Point, action: impl FnOnce(T) -> T) -> Result<()> {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let point = p as *mut T;
    check_ptr(point)?;
    let status_use = unsafe {
        point
            .as_mut()
            .ok_or_else(|| anyhow!("read pointer error: ({})", p))
    }?;
    let result = catch_unwind(AssertUnwindSafe(|| {
        replace_with_or_abort(status_use, action);
    }));
    match result {
        Ok(_) => Ok(()),
        Err(_) => throw("replace status error"),
    }
}

#[inline]
pub fn to_status<T>(p: Point) -> Result<Box<T>> {
    let point = p as *mut T;
    check_ptr(point)?;
    unsafe {
        if let None = point.as_ref() {
            Err(anyhow!("read pointer error: ({})", p))
        } else {
            Ok(Box::from_raw(point))
        }
    }
}
