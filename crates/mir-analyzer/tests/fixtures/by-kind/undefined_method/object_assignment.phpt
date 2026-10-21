===description===
Object assignment
===file===
<?php
class A {}
function f(A $a): void {
    $a["b"] = 1;
}
