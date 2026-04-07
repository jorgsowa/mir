===source===
<?php
function test($x): bool {
    return $x instanceof NoSuchClass;
}
===expect===
UndefinedClass: NoSuchClass
