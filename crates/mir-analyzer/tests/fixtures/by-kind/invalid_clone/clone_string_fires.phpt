===description===
InvalidClone fires when cloning a string parameter.
===config===
suppress=UnusedVariable
===file===
<?php
function f(string $s): void {
    clone $s;
//  ^^^^^^^^ InvalidClone: cannot clone non-object string
}
===expect===
