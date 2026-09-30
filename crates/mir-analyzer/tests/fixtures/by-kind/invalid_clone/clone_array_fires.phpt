===description===
InvalidClone fires when cloning an array parameter.
===config===
suppress=UnusedVariable
===file===
<?php
function f(array $a): void {
    clone $a;
//  ^^^^^^^^ InvalidClone: cannot clone non-object array
}
===expect===
