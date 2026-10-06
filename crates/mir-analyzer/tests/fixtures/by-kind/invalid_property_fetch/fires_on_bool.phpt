===description===
InvalidPropertyFetch fires on bool type.
===file===
<?php
/** @var bool $flag */
$flag = true;
$flag->foo;
//<^^^^^^^^^^ InvalidPropertyFetch: Cannot fetch property on non-object type 'bool'
