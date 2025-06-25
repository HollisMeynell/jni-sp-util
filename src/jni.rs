use jni::{
    JNIEnv,
    objects::{
        GlobalRef, JClass, JFieldID, JMethodID, JObject, JStaticFieldID, JStaticMethodID, JValueGen,
    },
    signature::{JavaType, ReturnType},
    sys::{jfieldID, jmethodID, jvalue},
};
use mini_moka::sync::Cache;
use std::fmt::Display;
use std::sync::LazyLock;

use crate::{error::Result, throw};

pub type ClassKey = i32;
pub type FieldKey = i32;
pub type StaticFieldKey = i32;
pub type MethodKey = i32;
pub type StaticMethodKey = i32;

pub static CLASS_CACHE: LazyLock<Cache<ClassKey, GlobalRef>> = LazyLock::new(|| Cache::new(30));
pub static FIELD_CACHE: LazyLock<Cache<FieldKey, usize>> = LazyLock::new(|| Cache::new(30));
pub static METHOD_CACHE: LazyLock<Cache<MethodKey, usize>> = LazyLock::new(|| Cache::new(30));
pub static STATIC_FIELD_CACHE: LazyLock<Cache<StaticFieldKey, usize>> =
    LazyLock::new(|| Cache::new(30));
pub static STATIC_METHOD_CACHE: LazyLock<Cache<StaticMethodKey, usize>> =
    LazyLock::new(|| Cache::new(30));

pub struct SpStaticField {
    cache: StaticFieldKey,
    name: Option<String>,
    ret: Option<String>,
}

impl SpStaticField {
    pub fn contains_cache(key: StaticFieldKey) -> bool {
        STATIC_FIELD_CACHE.contains_key(&key)
    }

    pub fn cache(key: StaticFieldKey) -> Self {
        Self {
            cache: key,
            name: None,
            ret: None,
        }
    }
    pub fn new(key: StaticFieldKey, name: &str, return_type: &SpType) -> Self {
        Self {
            cache: key,
            name: Some(name.to_string()),
            ret: Some(return_type.to_string()),
        }
    }

    pub fn init(&self, env: &mut JNIEnv, jclass: &JClass) -> Result<()> {
        if STATIC_FIELD_CACHE.contains_key(&self.cache) {
            return Ok(());
        }
        let raw_id = match (&self.name, &self.ret) {
            (Some(name), Some(sig)) => env.get_static_field_id(jclass, name, sig)?.into_raw(),
            _ => return throw("init static field error: name or return type is null"),
        };
        STATIC_FIELD_CACHE.insert(self.cache, raw_id as usize);
        Ok(())
    }

    pub fn call<'s>(
        &self,
        env: &'s mut JNIEnv,
        class: &JClass,
        ret: JavaType,
    ) -> Result<JValueGen<JObject<'s>>> {
        let field_id = match FIELD_CACHE.get(&self.cache) {
            Some(id) => unsafe { JStaticFieldID::from_raw(id as jfieldID) },
            None => return throw("no method cache"),
        };
        let result = env.get_static_field_unchecked(class, field_id, ret)?;
        Ok(result)
    }
}

pub struct SpField {
    cache: FieldKey,
    name: Option<String>,
    ret: Option<String>,
}

impl SpField {
    pub fn contains_cache(key: FieldKey) -> bool {
        FIELD_CACHE.contains_key(&key)
    }

    pub fn cache(key: FieldKey) -> Self {
        Self {
            cache: key,
            name: None,
            ret: None,
        }
    }

    pub fn new(key: FieldKey, name: &str, return_type: &SpType) -> Self {
        Self {
            cache: key,
            name: Some(name.to_string()),
            ret: Some(return_type.to_string()),
        }
    }

    pub fn init(&self, env: &mut JNIEnv, jclass: &JClass) -> Result<()> {
        if FIELD_CACHE.contains_key(&self.cache) {
            return Ok(());
        }
        let raw_id = match (&self.name, &self.ret) {
            (Some(name), Some(sig)) => env.get_field_id(jclass, name, sig)?.into_raw(),
            _ => return throw("init field error: name or return type is null"),
        };
        FIELD_CACHE.insert(self.cache, raw_id as usize);
        Ok(())
    }

    pub fn call<'s>(
        &self,
        env: &'s mut JNIEnv,
        this: &JObject,
        ret: ReturnType,
    ) -> Result<JValueGen<JObject<'s>>> {
        let field_id = match FIELD_CACHE.get(&self.cache) {
            Some(id) => unsafe { JFieldID::from_raw(id as jfieldID) },
            None => return throw("no method cache"),
        };
        let result = env.get_field_unchecked(this, field_id, ret)?;
        Ok(result)
    }
}

pub struct SpStaticMethod {
    cache: StaticMethodKey,
    name: Option<String>,
    sig: Option<String>,
}

impl SpStaticMethod {
    pub fn contains_cache(key: StaticMethodKey) -> bool {
        STATIC_METHOD_CACHE.contains_key(&key)
    }

    pub fn cache(key: StaticMethodKey) -> Self {
        Self {
            cache: key,
            name: None,
            sig: None,
        }
    }

    pub fn new(key: StaticMethodKey, name: &str, return_type: &SpType, args: &[SpType]) -> Self {
        let mut all_len = return_type.get_str_len() + 2;
        for n in args {
            all_len += n.get_str_len();
        }
        let mut sig_builder = String::with_capacity(all_len);
        sig_builder.push('(');
        for n in args {
            sig_builder.push_str(&n.to_string());
        }
        sig_builder.push(')');
        sig_builder.push_str(&return_type.to_string());

        Self {
            cache: key,
            name: Some(name.to_string()),
            sig: Some(sig_builder),
        }
    }

    pub fn init(&self, env: &mut JNIEnv, jclass: &JClass) -> Result<()> {
        if STATIC_METHOD_CACHE.contains_key(&self.cache) {
            return Ok(());
        }
        let raw_id = match (&self.name, &self.sig) {
            (Some(name), Some(sig)) => env.get_static_method_id(jclass, name, sig)?.into_raw(),
            _ => return throw("init static method error: name or sig is null"),
        };
        STATIC_METHOD_CACHE.insert(self.cache, raw_id as usize);
        Ok(())
    }

    pub fn call<'s>(
        &self,
        env: &'s mut JNIEnv,
        class: &JClass,
        args: &[jvalue],
        ret: ReturnType,
    ) -> Result<JValueGen<JObject<'s>>> {
        let method_id = match STATIC_METHOD_CACHE.get(&self.cache) {
            Some(id) => unsafe { JStaticMethodID::from_raw(id as jmethodID) },
            None => return throw("no method cache"),
        };
        let result = unsafe { env.call_static_method_unchecked(class, method_id, ret, args)? };
        Ok(result)
    }
}

pub struct SpMethod {
    cache: MethodKey,
    name: Option<String>,
    sig: Option<String>,
}

impl SpMethod {
    pub fn contains_cache(key: MethodKey) -> bool {
        METHOD_CACHE.contains_key(&key)
    }

    pub fn cache(key: MethodKey) -> Self {
        Self {
            cache: key,
            name: None,
            sig: None,
        }
    }
    pub fn new(key: MethodKey, name: &str, return_type: &SpType, args: &[SpType]) -> Self {
        let mut all_len = return_type.get_str_len() + 2;
        for n in args {
            all_len += n.get_str_len();
        }
        let mut sig_builder = String::with_capacity(all_len);
        sig_builder.push('(');
        for n in args {
            sig_builder.push_str(&n.to_string());
        }
        sig_builder.push(')');
        sig_builder.push_str(&return_type.to_string());

        Self {
            cache: key,
            name: Some(name.to_string()),
            sig: Some(sig_builder),
        }
    }

    pub fn init(&self, env: &mut JNIEnv, jclass: &JClass) -> Result<()> {
        if METHOD_CACHE.contains_key(&self.cache) {
            return Ok(());
        }
        let raw_id = match (&self.name, &self.sig) {
            (Some(name), Some(sig)) => env.get_method_id(jclass, name, sig)?.into_raw(),
            _ => return throw("init static method error: name or sig is null"),
        };
        METHOD_CACHE.insert(self.cache, raw_id as usize);
        Ok(())
    }

    pub fn call<'s>(
        &self,
        env: &'s mut JNIEnv,
        this: &JObject,
        args: &[jvalue],
        ret: ReturnType,
    ) -> Result<JValueGen<JObject<'s>>> {
        let method_id = match METHOD_CACHE.get(&self.cache) {
            Some(id) => unsafe { JMethodID::from_raw(id as jmethodID) },
            None => return throw("no method cache"),
        };
        let result = unsafe { env.call_method_unchecked(this, method_id, ret, args)? };
        Ok(result)
    }
}

pub enum SpType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Sort,
    Boolean,
    Void,
    Class(SpClass),
    Array(Box<SpType>),
}

impl SpType {
    pub fn new_class(class: &str) -> Self {
        let c = SpClass::from_sig(class);
        Self::Class(c)
    }

    pub fn get_str_len(&self) -> usize {
        match self {
            Self::Class(class) => {
                let len = class
                    .class_full_path
                    .as_deref()
                    .unwrap_or("java/lang/Object")
                    .len();
                len + 2
            }
            Self::Array(class) => &class.get_str_len() + 1,
            _ => 1,
        }
    }
}

impl Default for SpType {
    fn default() -> Self {
        Self::Void
    }
}

impl Display for SpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Byte => f.write_str("B"),
            Self::Char => f.write_str("C"),
            Self::Double => f.write_str("D"),
            Self::Float => f.write_str("F"),
            Self::Int => f.write_str("I"),
            Self::Long => f.write_str("L"),
            Self::Sort => f.write_str("S"),
            Self::Boolean => f.write_str("Z"),
            Self::Void => f.write_str("V"),
            Self::Class(class) => write!(
                f,
                "L{};",
                class
                    .class_full_path
                    .as_deref()
                    .unwrap_or("java/lang/Object")
            ),
            Self::Array(t) => write!(f, "[{}", t.to_string()),
        }
    }
}

pub struct SpClass {
    cache: ClassKey,
    class_full_path: Option<String>,
    jni_class_ref: Option<GlobalRef>,
}

impl SpClass {
    pub fn contains_cache(key: ClassKey) -> bool {
        CLASS_CACHE.contains_key(&key)
    }

    pub fn from_sig(sig: &str) -> Self {
        let path = sig.replace(".", "/");
        Self {
            cache: -1,
            class_full_path: Some(path),
            jni_class_ref: None,
        }
    }

    pub fn cache(key: ClassKey) -> Self {
        Self {
            cache: key,
            class_full_path: None,
            jni_class_ref: None,
        }
    }
    pub fn new(key: ClassKey, sig: &str) -> Self {
        let path = sig.replace(".", "/");
        Self {
            cache: key,
            class_full_path: Some(path),
            jni_class_ref: None,
        }
    }

    pub fn init(&mut self, env: &mut JNIEnv) -> Result<()> {
        if self.jni_class_ref.is_some() {
            return Ok(());
        }
        if self.cache < 0 {
            if self.class_full_path.is_none() {
                return throw("no class");
            }
        } else if self.jni_class_ref.is_some() {
            return Ok(());
        }

        let result = match CLASS_CACHE.get(&self.cache) {
            Some(global_ref) => global_ref,
            None => {
                let sig = match &self.class_full_path {
                    Some(name) => name,
                    None => return throw("no class cache"),
                };
                let class = env.find_class(sig)?;
                let raw = env.new_global_ref(class)?;
                CLASS_CACHE.insert(self.cache, raw.clone());
                raw
            }
        };
        self.jni_class_ref = Some(result);
        Ok(())
    }

    pub fn get_jni_class(&self) -> Result<&JClass> {
        match &self.jni_class_ref {
            Some(class_ref) => Ok(<&JClass>::from(class_ref.as_obj())),
            None => throw("class not init"),
        }
    }
}

/// example:
/// - class: `class: env, key, "java.lang.Object"`
/// - static_field: `static_field: env, key, &JClass, "name", &SpType`
/// - field: `field: env, key, &JClass, "name", &SpType`
/// - static_method: `static_method: env, key, &JClass, "name", &SpType, &[SpType] `
/// - method: `method: env, key, &JClass, "name", &SpType, &[SpType] `
#[macro_export]
macro_rules! get_sp_struct {
    (class: $env:expr, $key:expr, $sig:expr) => {
        if SpClass::contains_cache($key) {
            let class = SpClass::cache($key);
            Ok(class)
        } else {
            let mut class = SpClass::new($key, $sig);
            match class.init($env) {
                Ok(_) => Ok(class),
                Err(e) => Err(e),
            }
        }
    };
    (static_field: $env:ident, $key:expr, $class:expr, $name:expr, $t:expr) => {
        if SpStaticField::contains_cache($key) {
            let field = SpStaticField::cache($key);
            Ok(field)
        } else {
            let field = SpStaticField::new($key, $name, $t);
            match field.init($env, $class) {
                Ok(_) => Ok(field),
                Err(e) => Err(e),
            }
        }
    };
    (field: $env:ident, $key:expr, $class:expr, $name:expr, $t:expr) => {
        if SpField::contains_cache($key) {
            let field = SpField::cache($key);
            Ok(field)
        } else {
            let field = SpField::new($key, $name, $t);
            match field.init($env, $class) {
                Ok(_) => Ok(field),
                Err(e) => Err(e),
            }
        }
    };
    (static_method: $env:ident, $key:expr, $class:expr, $name:expr, $ret:expr, $args:expr) => {
        if SpStaticMethod::contains_cache($key) {
            let method = SpStaticMethod::cache($key);
            Ok(method)
        } else {
            let method = SpStaticMethod::new($key, $name, $ret, $args);
            match method.init($env, $class) {
                Ok(_) => Ok(method),
                Err(e) => Err(e),
            }
        }
    };
    (method: $env:ident, $key:expr, $class:expr, $name:expr, $ret:expr, $args:expr) => {
        if SpMethod::contains_cache($key) {
            let method = SpMethod::cache($key);
            Ok(method)
        } else {
            let method = SpMethod::new($key, $name, $ret, $args);
            match method.init($env, $class) {
                Ok(_) => Ok(method),
                Err(e) => Err(e),
            }
        }
    };
}
