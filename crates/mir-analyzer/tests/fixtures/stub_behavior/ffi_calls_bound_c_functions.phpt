===description===
Calls and variable reads on an FFI instance reach the bound C symbols
===file===
<?php
function page(): mixed {
    $ffi = \FFI::cdef('size_t zend_get_page_size(void); int errno;');
    return [$ffi->zend_get_page_size(), $ffi->errno];
}
