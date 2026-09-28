===description===
Editing a method's return type re-checks callers in other files.
===file:Box.php===
<?php
class Box { public function get(): int { return 1; } }
===file:Use.php===
<?php
function run(Box $b): int { return $b->get(); }
===expect===
<<none>>
===edit:Box.php===
<?php
class Box { public function get(): string { return ''; } }
===expect===
Use.php: InvalidReturnType@2:28-2:45: Return type 'string' is not compatible with declared 'int'
