===description===
instanceof unknown class
===config===
suppress=MissingParamType
===file===
<?php
function test($x): bool {
    return $x instanceof NoSuchClass;
//                       ^^^^^^^^^^^ UndefinedClass: Class NoSuchClass does not exist
}
===expect===
