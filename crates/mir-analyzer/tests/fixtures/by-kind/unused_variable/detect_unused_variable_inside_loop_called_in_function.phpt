===description===
Detect unused variable inside loop called in function
===file===
<?php
function foo(int $s) : int {
    return $s;
}

function bar() : void {
    foreach ([1, 2, 3] as $i) {
        $i = foo($i);
    }
}
===expect===
UnusedVariable@8:9-8:11: Variable $i is never read
