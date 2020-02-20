/// Pointer type for Js allocated object
/// Consider this as a Box, but pointing into JS memory
#[repr(transparent)]
pub struct JsObject(u32);

impl JsObject {
    /// Get a weak reference to js memory
    /// No guarantees against js garbage collector 
    pub fn weak(&self) -> JsObjectWeak {
        JsObjectWeak(self.0)
    }
}
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct JsObjectWeak(u32);


impl Drop for JsObject {
    fn drop(&mut self) {
        unsafe { js_free_object(self.weak()); }
    }
}

/// Private unsafe JS api
extern "C" {
    /// Allocate new js object with data proveden. 
    /// Returned JsObject is safe to use and will follow usual JsObject ownership rules
    fn js_create_string(buf: *const u8, max_len: u32) -> JsObject;

    /// Allocate new empty js object. Like "var obj = {};".
    /// Returned JsObject is safe to use and will follow usual JsObject ownership rules
    fn js_create_object() -> JsObject;

    /// This will not delete or delallocate JS object, but will stop saving it from JS garbage collector
    fn js_free_object(js_object: JsObjectWeak);

    /// Will read object byte by byte into rust memory assuming that object is an array
    fn js_unwrap_to_str(js_object: JsObjectWeak, buf: *mut u8, max_len: u32);

    // Will panic if js_object is not a string
    // Will calculate the length of string bytes representation
    fn js_string_length(js_object: JsObjectWeak) -> u32;
    
    /// Get .field or ["field"] of given JsObject
    fn js_field(js_object: JsObjectWeak, buf: *mut u8, len: u32) -> JsObject;

    /// Set .field or ["field"] to given f32, like "object.field = data";
    fn js_set_field_f32(js_object: JsObjectWeak, buf: *mut u8, len: u32, data: f32);

}

impl JsObject {
    /// Allocate new javascript object with string type 
    pub fn string(string: &str) -> JsObject {
        unsafe { js_create_string(string.as_ptr() as _, string.len() as _) }
    }

    /// Allocate new javascript object with object type. Like "var object = {}" in JS.
    pub fn object() -> JsObject {
        unsafe { js_create_object() }
    }

    /// Read js object to given string
    /// Will not allocate memory is string is large enough, will use "String::reserve" otherwise
    /// Will panic if object is not a string
    pub fn to_string(&self, buf: &mut String) {
        let len = unsafe { js_string_length(self.weak()) };

        if len as usize > buf.len() {
            buf.reserve(len as usize - buf.len());
        }
        unsafe { buf.as_mut_vec().set_len(len as usize) };
        unsafe { js_unwrap_to_str(self.weak(), buf.as_mut_vec().as_mut_ptr(), len as u32) };
    }

    /// Get a new JsObject from this object .field
    /// Will panic if self is not an object or map
    pub fn field(&self, field: &str) -> JsObject {
        unsafe { js_field(self.weak(), field.as_ptr() as _, field.len() as _) }
    }

    /// Set .field or ["field"] to given f32, like "object.field = data";
    /// Will panic if self is not an object or map
    pub fn set_field_f32(&self, field: &str, data: f32) {
        unsafe { js_set_field_f32(self.weak(), field.as_ptr() as _, field.len() as _, data) }
    }

}
