===description===
Editing a callee's return type re-checks its unchanged caller.
===file:Lib.php===
<?php
function lib(): int { return 1; }
===file:Use.php===
<?php
function run(): int { return lib(); }
===edit:Lib.php===
<?php
function lib(): string { return ''; }
===expect===
Use.php: InvalidReturnType@2:22-2:35: Return type 'string' is not compatible with declared 'int'
