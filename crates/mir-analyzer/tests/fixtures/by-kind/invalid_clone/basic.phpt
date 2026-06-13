===description===
InvalidClone fires when clone is used on a non-object type.
===file===
<?php
$x = 42;
$y = clone $x;
===expect===
InvalidClone@3:6-3:14: cannot clone non-object 42
