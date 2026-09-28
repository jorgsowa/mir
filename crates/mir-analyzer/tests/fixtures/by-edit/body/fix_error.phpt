===description===
Fixing an error in the edited file clears it.
===file:Lib.php===
<?php
function lib(): int { return 1; }
===file:Use.php===
<?php
function run(): string { return lib(); }
===expect===
Use.php: InvalidReturnType@2:25-2:38: Return type 'int' is not compatible with declared 'string'
===edit:Use.php===
<?php
function run(): int { return lib(); }
===expect===
<<none>>
