===description===
instanceof unknown class
===file===
<?php
function test($x): bool {
    return $x instanceof NoSuchClass;
}
===expect===
UndefinedClass@3:26: Class NoSuchClass does not exist
