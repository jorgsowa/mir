===description===
Object assignment
===file===
<?php
class A {}
function f(A $a): A {
    $a["b"] = 1;
    return $a;
}
