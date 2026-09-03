===description===
FP-A: extension_loaded() guard works with multiple extensions combined via &&,
and with nested guards. The else branch does NOT carry the guard.
===config===
php_version=8.2
suppress=UnusedVariable
===file===
<?php

// Multiple extensions combined — both guards active
if (extension_loaded('ext_a') && extension_loaded('ext_b')) {
    new \ExtA\PrimaryClass();
    new \ExtB\SecondaryClass();
}

// Nested extension_loaded guards
if (extension_loaded('outer_ext')) {
    $obj = new \OuterExtClass();
    if (extension_loaded('inner_ext')) {
        new \InnerExtClass();
    }
}

// extension_loaded combined with class_exists
if (extension_loaded('optional') && class_exists(\OptionalClass::class)) {
    $x = new \OptionalClass();
}
===expect===
