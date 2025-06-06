use jni::{
    JNIEnv,
    objects::{GlobalRef, JClass, JFieldID},
    sys::jfieldID,
};
use mini_moka::sync::Cache;
use std::{fmt::format, sync::LazyLock};

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

#[inline]
pub fn get_jni_field_id(key: FieldKey, _env: &JNIEnv) -> Result<JFieldID> {
    let cache_value = FIELD_CACHE.get(&key);
    if let Some(id) = cache_value {
        let result = unsafe {
            let id = id as jfieldID;
            JFieldID::from_raw(id)
        };
        return Ok(result);
    }

    throw("no cache")
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

impl ToString for SpType {
    fn to_string(&self) -> String {
        match self {
            Self::Byte => "B".to_string(),
            Self::Char => "C".to_string(),
            Self::Double => "D".to_string(),
            Self::Float => "F".to_string(),
            Self::Int => "I".to_string(),
            Self::Long => "L".to_string(),
            Self::Sort => "S".to_string(),
            Self::Boolean => "Z".to_string(),
            Self::Void => "V".to_string(),
            Self::Class(class) => format!("L{};", class.class_full_path.as_deref().unwrap_or("java/lang/Object")),
            Self::Array(t) => format!("[{}", t.to_string()),
        }
    }
}



pub struct SpClass {
    cache: ClassKey,
    class_full_path: Option<String>,
    jni_class_ref: Option<GlobalRef>,
}

impl SpClass {
    pub fn from_sig(sig: &str) -> Self {
        let path = sig.replace(".", "/");
        Self {
            cache: -1,
            class_full_path: Some(path),
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
        if self.cache < 0 && self.class_full_path.is_none() {
            return throw("no class");
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
