===description===
Changing a property's declared type re-checks reads in other files.
===file:Box.php===
<?php
class Box { public int $v = 1; }
===file:Use.php===
<?php
function run(Box $b): int { return $b->v; }
===expect===
===edit:Box.php===
<?php
class Box { public string $v = ''; }
===expect===
Use.php: InvalidReturnType@2:28-2:41: Return type 'string' is not compatible with declared 'int'
