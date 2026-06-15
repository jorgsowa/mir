===description===
Passing an integer where string is expected is allowed in non-strict mode (PHP coerces via (string)$int)
===file===
<?php
function greet(string $name): void { echo $name; }
function label(string $prefix, string $suffix): string { return $prefix . $suffix; }

// Should NOT report InvalidArgument — PHP coerces int to string in coercive mode.
greet(42);
greet(0);
label('item_', 7);
===expect===
