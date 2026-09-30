===description===
InvalidPropertyFetch fires on array type.
===file===
<?php
/** @var array<int, string> $items */
$items = [];
$items->foo;
//<^^^^^^^^^^^ InvalidPropertyFetch: Cannot fetch property on non-object type 'array<int, string>'
===expect===
