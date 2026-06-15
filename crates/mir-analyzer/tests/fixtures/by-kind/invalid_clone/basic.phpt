===description===
InvalidClone fires when clone is used on a non-object type.
===config===
suppress=UnusedVariable
===file===
<?php
$x = 42;
$y = clone $x;
===expect===
InvalidClone@3:5-3:13: cannot clone non-object 42
