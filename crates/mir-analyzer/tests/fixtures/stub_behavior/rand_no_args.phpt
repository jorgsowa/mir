===description===
rand can be called with zero arguments
===file===
<?php
// rand with no arguments is allowed (returns random int in full range)
$r = rand();
// rand with two arguments is also allowed
$r2 = rand(1, 100);
===expect===
