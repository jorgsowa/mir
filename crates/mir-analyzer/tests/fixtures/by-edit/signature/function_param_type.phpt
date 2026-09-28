===description===
Narrowing a callee's parameter type flags the unchanged call site.
===file:Lib.php===
<?php
function lib(int $n): int { return $n; }
===file:Use.php===
<?php
function run(): void { lib(1); }
===expect===
===edit:Lib.php===
<?php
function lib(string $n): string { return $n; }
===expect===
Use.php: ArgumentTypeCoercion@2:27-2:28: Argument $n of lib() expects 'string', got '1' — coercion may fail at runtime
