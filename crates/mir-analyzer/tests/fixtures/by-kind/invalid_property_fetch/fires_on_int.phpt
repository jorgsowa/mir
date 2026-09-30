===description===
InvalidPropertyFetch fires on int type.
===file===
<?php
/** @var int $x */
$x = 5;
$x->foo;
//<^^^^^^^ InvalidPropertyFetch: Cannot fetch property on non-object type 'int'
===expect===
