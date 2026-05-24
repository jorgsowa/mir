===description===
functionWithVar
===file===
<?php
function test() {
    /** @var mixed $a */
    $a = 5;
    clone $a;
}
===expect===
MixedClone@5:5: cannot clone mixed
