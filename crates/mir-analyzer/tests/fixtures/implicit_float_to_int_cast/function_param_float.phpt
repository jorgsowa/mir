===description===
Passing float value to float-typed parameter - should not emit

===file===
<?php
function foo(float $n): void {
    echo $n;
}

$x = 3.7;
foo($x);

===expect===
