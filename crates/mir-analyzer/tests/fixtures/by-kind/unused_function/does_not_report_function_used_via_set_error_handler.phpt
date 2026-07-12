===description===
a function used only as a bare string callback to set_error_handler must not be reported unused
===config===
suppress=
===file===
<?php
function myHandler(int $errno, string $errstr): bool {
    echo $errno, $errstr;
    return true;
}

set_error_handler('myHandler');
===expect===
