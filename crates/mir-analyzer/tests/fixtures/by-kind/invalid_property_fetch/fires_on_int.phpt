===description===
InvalidPropertyFetch fires on int type.
===file===
<?php
/** @var int $x */
$x = 5;
$x->foo;
===expect===
InvalidPropertyFetch@4:0-4:7: Cannot fetch property on non-object type 'int'
