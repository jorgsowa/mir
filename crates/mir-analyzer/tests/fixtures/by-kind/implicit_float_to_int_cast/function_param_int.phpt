===description===
Passing int value to int-typed parameter - should not emit

===file===
<?php
function foo(int $n): void {
    echo $n;
}

$x = 3;
foo($x);

===expect===
