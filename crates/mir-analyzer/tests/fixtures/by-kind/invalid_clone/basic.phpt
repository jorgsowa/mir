===description===
InvalidClone fires when clone is used on a non-object type.
===config===
suppress=UnusedVariable
===file===
<?php
$x = 42;
$y = clone $x;
//   ^^^^^^^^ InvalidClone: cannot clone non-object 42
===expect===
