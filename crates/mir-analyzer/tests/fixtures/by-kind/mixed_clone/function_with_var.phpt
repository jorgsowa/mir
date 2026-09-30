===description===
Function with var
===config===
suppress=MissingReturnType
===file===
<?php
function test() {
    /** @var mixed $a */
    $a = 5;
    clone $a;
//  ^^^^^^^^ MixedClone: cannot clone mixed
}
===expect===
