===description===
mt_rand can be called with zero arguments
===config===
suppress=UnusedVariable
===file===
<?php
// mt_rand with no arguments is allowed (returns random int in full range)
$r = mt_rand();
// mt_rand with two arguments is also allowed
$r2 = mt_rand(1, 100);
===expect===
