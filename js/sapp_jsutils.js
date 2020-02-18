var ctx = null;

js_objects = {}
unique_js_id = 0

register_plugin = function (importObject) {
    importObject.env.js_create_string = function (buf, max_len) {
        var string = UTF8ToString(buf, max_len);
        return js_object(string);
    }

    importObject.env.js_unwrap_to_str = function (js_object, buf, max_len) {
        var str = js_objects[js_object];
        var utf8array = toUTF8Array(str);
        var length = utf8array.length;
        var dest = new Uint8Array(wasm_memory.buffer, buf, max_len); // with max_len in case of buffer overflow we will panic (I BELIEVE) in js, no UB in rust
        for (var i = 0; i < length; i++) {
            dest[i] = utf8array[i];
        }
    }

    // measure length of the string. This function allocates because there is no way
    // go get string byte length in JS 
    importObject.env.js_string_length = function (js_object) {
        var str = js_objects[js_object];
        return toUTF8Array(str).length;
    }

    importObject.env.js_free_object = function (js_object) {
        delete js_objects[js_object];
    }

    importObject.env.js_field = function (js_object, buf, length) {
        // UTF8ToString is from gl.js wich should be in the scope now
        var field_name = UTF8ToString(buf, length);

        // apparently .field and ["field"] is the same thing in js
        var field = js_objects[js_object][field_name];

        var id = unique_js_id
        js_objects[id] = field

        unique_js_id += 1;

        return id;
    }
}
miniquad_add_plugin({ register_plugin });

// Its like https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder, 
// but works on more browsers
function toUTF8Array(str) {
    var utf8 = [];
    for (var i = 0; i < str.length; i++) {
        var charcode = str.charCodeAt(i);
        if (charcode < 0x80) utf8.push(charcode);
        else if (charcode < 0x800) {
            utf8.push(0xc0 | (charcode >> 6),
                0x80 | (charcode & 0x3f));
        }
        else if (charcode < 0xd800 || charcode >= 0xe000) {
            utf8.push(0xe0 | (charcode >> 12),
                0x80 | ((charcode >> 6) & 0x3f),
                0x80 | (charcode & 0x3f));
        }
        // surrogate pair
        else {
            i++;
            // UTF-16 encodes 0x10000-0x10FFFF by
            // subtracting 0x10000 and splitting the
            // 20 bits of 0x0-0xFFFFF into two halves
            charcode = 0x10000 + (((charcode & 0x3ff) << 10)
                | (str.charCodeAt(i) & 0x3ff))
            utf8.push(0xf0 | (charcode >> 18),
                0x80 | ((charcode >> 12) & 0x3f),
                0x80 | ((charcode >> 6) & 0x3f),
                0x80 | (charcode & 0x3f));
        }
    }
    return utf8;
}

// Store js object reference to prevent JS garbage collector on destroying it
// And let Rust keep ownership of this reference
// There is no guarantees on JS side of this reference uniqueness, its good idea to use this only on rust functions arguments
function js_object(obj) {
    var id = unique_js_id;

    js_objects[id] = obj;
    unique_js_id += 1;
    return id;
}

/// Get the real object from JsObject returned from rust
function from_js_object(id) {
    return js_objects[id];
}
