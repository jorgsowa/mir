===description===
Implicit nullability adds only null: other wrong argument types are still reported.
===config===
suppress=UnusedParam
===file===
<?php
/** @param string $f */
function plain($f = null): void {}
plain([]);
===expect===
InvalidArgument@4:6-4:8: Argument $f of plain() expects 'string|null', got 'array{}'
