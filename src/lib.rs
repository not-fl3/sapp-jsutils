#[no_mangle]
pub extern "C" fn sapp_jsutils_crate_version() -> u32 {
    let major = dbg!(env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap());
    let minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap();
    let patch = dbg!(env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap());

    (major << 24) + (minor << 16) + patch
}

/// Pointer type for Js allocated object
/// Consider this as a Box, but pointing into JS memory
/// -1 is nil
#[repr(transparent)]
pub struct JsObject(i32);

impl JsObject {
    /// Get a weak reference to js memory
    /// No guarantees against js garbage collector
    pub fn weak(&self) -> JsObjectWeak {
        JsObjectWeak(self.0)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct JsObjectWeak(i32);

impl Drop for JsObject {
    fn drop(&mut self) {
        unsafe {
            js_free_object(self.weak());
        }
    }
}

/// Private unsafe JS api
extern "C" {
    /// Allocate new js object with data providen.
    /// Returned JsObject is safe to use and will follow usual JsObject ownership rules
    fn js_create_string(buf: *const u8, max_len: u32) -> JsObject;

    /// Same as create_string, on both api and implementation, however will not perform UTF8 conversion on the JS side.
    fn js_create_buffer(buf: *const u8, max_len: u32) -> JsObject;

    /// Allocate new empty js object. Like "var obj = {};".
    /// Returned JsObject is safe to use and will follow usual JsObject ownership rules
    fn js_create_object() -> JsObject;

    /// This will not delete or delallocate JS object, but will stop saving it from JS garbage collector
    fn js_free_object(js_object: JsObjectWeak);

    /// Will read object byte by byte into rust memory assuming that object is an array
    fn js_unwrap_to_str(js_object: JsObjectWeak, buf: *mut u8, max_len: u32);

    /// Will read object byte by byte into rust memory assuming that object is an array
    fn js_unwrap_to_buf(js_object: JsObjectWeak, buf: *mut u8, max_len: u32);

    // Will panic if js_object is not a string
    // Will calculate the length of string bytes representation
    fn js_string_length(js_object: JsObjectWeak) -> u32;
    fn js_buf_length(js_object: JsObjectWeak) -> u32;

    /// .field or ["field"] == undefined
    fn js_have_field(js_object: JsObjectWeak, buf: *mut u8, len: u32) -> bool;

    /// Get .field or ["field"] of given JsObject
    fn js_field(js_object: JsObjectWeak, buf: *mut u8, len: u32) -> JsObject;

    /// Get a numerical value of .field or ["field"] of given JsObject
    fn js_field_f32(js_object: JsObjectWeak, buf: *mut u8, len: u32) -> f32;

    /// Get a u32 value of .field or ["field"] of given JsObject
    fn js_field_u32(js_object: JsObjectWeak, buf: *mut u8, len: u32) -> u32;

    /// Set .field or ["field"] to given string, like "object.field = "data"";
    fn js_set_field_string(
        js_object: JsObjectWeak,
        buf: *mut u8,
        len: u32,
        data_buf: *mut u8,
        data_len: u32,
    );

    /// Set .field or ["field"] to given f32, like "object.field = data";
    fn js_set_field_f32(js_object: JsObjectWeak, buf: *mut u8, len: u32, data: f32);

    /// Set .field or ["field"] to given u32, like "object.field = data";
    fn js_set_field_u32(js_object: JsObjectWeak, buf: *mut u8, len: u32, data: u32);

}

impl JsObject {
    /// Allocate new javascript object with string type
    pub fn string(string: &str) -> JsObject {
        unsafe { js_create_string(string.as_ptr() as _, string.len() as _) }
    }

    /// Allocate new javascript object with Uint8Array type
    pub fn buffer(data: &[u8]) -> JsObject {
        unsafe { js_create_buffer(data.as_ptr() as _, data.len() as _) }
    }

    /// Allocate new javascript object with object type. Like "var object = {}" in JS.
    pub fn object() -> JsObject {
        unsafe { js_create_object() }
    }

    /// Read JS object content to a given string
    /// Will panic if object is not a string
    /// Will not allocate memory if string is large enough, will use "String::reserve" otherwise
    pub fn to_string(&self, buf: &mut String) {
        let len = unsafe { js_string_length(self.weak()) };

        if len as usize > buf.len() {
            buf.reserve(len as usize - buf.len());
        }
        unsafe { buf.as_mut_vec().set_len(len as usize) };
        unsafe { js_unwrap_to_str(self.weak(), buf.as_mut_vec().as_mut_ptr(), len as u32) };
    }

    /// Read JS object content to a given bytes buffer
    /// Will panic if object is not a buffer
    /// Will use .resize() on "buf", so if "buf" is large enough - no memory is going to be allocated here
    pub fn to_byte_buffer(&self, buf: &mut Vec<u8>) {
        let len = unsafe { js_buf_length(self.weak()) };
        buf.resize(len as usize, 0u8);
        unsafe { js_unwrap_to_buf(self.weak(), buf.as_mut_ptr(), len as u32) };
    }

    /// Get a new JsObject from this object .field
    /// Will panic if self is not an object or map
    pub fn field(&self, field: &str) -> JsObject {
        unsafe { js_field(self.weak(), field.as_ptr() as _, field.len() as _) }
    }

    /// Get a value from this object .field
    /// Will panic if self is not an object or map
    pub fn field_u32(&self, field: &str) -> u32 {
        unsafe { js_field_u32(self.weak(), field.as_ptr() as _, field.len() as _) }
    }

    /// .field == undefined
    pub fn have_field(&self, field: &str) -> bool {
        unsafe { js_have_field(self.weak(), field.as_ptr() as _, field.len() as _) }
    }

    /// Get a value from this object .field
    /// Will panic if self is not an object or map
    pub fn field_f32(&self, field: &str) -> f32 {
        unsafe { js_field_f32(self.weak(), field.as_ptr() as _, field.len() as _) }
    }

    /// Set .field or ["field"] to given f32, like "object.field = data";
    /// Will panic if self is not an object or map
    pub fn set_field_f32(&self, field: &str, data: f32) {
        unsafe { js_set_field_f32(self.weak(), field.as_ptr() as _, field.len() as _, data) }
    }

    /// Set .field or ["field"] to given u32, like "object.field = data";
    /// Will panic if self is not an object or map
    pub fn set_field_u32(&self, field: &str, data: u32) {
        unsafe { js_set_field_u32(self.weak(), field.as_ptr() as _, field.len() as _, data) }
    }

    /// Set .field or ["field"] to given string, like "object.field = data";
    /// Will panic if self is not an object or map

    pub fn set_field_string(&self, field: &str, data: &str) {
        unsafe {
            js_set_field_string(
                self.weak(),
                field.as_ptr() as _,
                field.len() as _,
                data.as_ptr() as _,
                data.len() as _,
            )
        }
    }

    /// JS function returning JsObject may return -1 instead.
    /// Those JsObject are considering nil and if function may do this JsObjects
    /// should be checked for nil.
    /// Unfortunately this is not typecheked by rust complier at all,
    /// so any "f() -> JsObject" function may return nil
    pub fn is_nil(&self) -> bool {
        self.0 == -1
    }
}
