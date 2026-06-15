===description===
instanceof unknown class
===config===
suppress=MissingParamType
===file===
<?php
function test($x): bool {
    return $x instanceof NoSuchClass;
}
===expect===
UndefinedClass@3:25-3:36: Class NoSuchClass does not exist
