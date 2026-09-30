===description===
InvalidClone fires when cloning a false literal (bool subtype).
===config===
suppress=UnusedVariable
===file===
<?php
$x = false;
clone $x;
//<^^^^^^^^ InvalidClone: cannot clone non-object false
===expect===
