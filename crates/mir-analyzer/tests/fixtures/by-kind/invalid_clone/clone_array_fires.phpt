===description===
InvalidClone fires when cloning an array parameter.
===config===
suppress=UnusedVariable
===file===
<?php
function f(array $a): void {
    clone $a;
}
===expect===
InvalidClone@3:4-3:12: cannot clone non-object array
