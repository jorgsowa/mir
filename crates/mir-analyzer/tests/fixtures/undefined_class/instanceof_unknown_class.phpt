===description===
instanceof unknown class
===file===
<?php
function test($x): bool {
    return $x instanceof NoSuchClass;
}
===expect===
UndefinedClass@3:25: Class NoSuchClass does not exist
===ignore===
TODO
