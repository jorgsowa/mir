===description===
InvalidPropertyFetch fires on bool type.
===file===
<?php
/** @var bool $flag */
$flag = true;
$flag->foo;
===expect===
InvalidPropertyFetch@4:0-4:10: Cannot fetch property on non-object type 'bool'
