===description===
A write to the variable a reference was taken from is read through the reference.
===file===
<?php
function f(): string {
    $b = 'x';
    $a = &$b;
    $b = 'c';
    return $a;
}
