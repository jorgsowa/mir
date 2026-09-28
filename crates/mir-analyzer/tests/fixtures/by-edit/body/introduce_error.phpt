===description===
An edit that introduces an error in the edited file itself is reported.
===file:Lib.php===
<?php
function lib(): int { return 1; }
===file:Use.php===
<?php
function run(): int { return lib(); }
===expect===
<<none>>
===edit:Use.php===
<?php
function run(): int { return 'x'; }
===expect===
Use.php: InvalidReturnType@2:22-2:33: Return type '"x"' is not compatible with declared 'int'
