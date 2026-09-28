===description===
Changing an inherited method's return type re-checks callers through the child.
===file:Base.php===
<?php
class Base { public function id(): int { return 1; } }
===file:Child.php===
<?php
class Child extends Base {}
===file:Use.php===
<?php
function run(Child $c): int { return $c->id(); }
===edit:Base.php===
<?php
class Base { public function id(): string { return ''; } }
===expect===
Use.php: InvalidReturnType@2:30-2:46: Return type 'string' is not compatible with declared 'int'
