===description===
InvalidPropertyFetch fires on array type.
===file===
<?php
/** @var array<int, string> $items */
$items = [];
$items->foo;
===expect===
InvalidPropertyFetch@4:0-4:11: Cannot fetch property on non-object type 'array<int, string>'
