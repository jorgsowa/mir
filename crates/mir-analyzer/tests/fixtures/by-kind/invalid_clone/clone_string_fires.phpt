===description===
InvalidClone fires when cloning a string parameter.
===config===
suppress=UnusedVariable
===file===
<?php
function f(string $s): void {
    clone $s;
}
===expect===
InvalidClone@3:4-3:12: cannot clone non-object string
