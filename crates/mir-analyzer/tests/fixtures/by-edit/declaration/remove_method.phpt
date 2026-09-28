===description===
Removing a method reports its callers as undefined method calls.
===file:Box.php===
<?php
class Box { public function get(): int { return 1; } }
===file:Use.php===
<?php
function run(Box $b): void { $b->get(); }
===expect===
<<none>>
===edit:Box.php===
<?php
class Box {}
===expect===
Use.php: UndefinedMethod@2:29-2:38: Method Box::get() does not exist
