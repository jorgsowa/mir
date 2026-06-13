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
}
===expect===
MixedClone@5:5-5:13: cannot clone mixed
