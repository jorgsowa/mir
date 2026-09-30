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
//      ^^ UnusedForeachValue: Foreach value $i is never read
    }
}
===expect===
