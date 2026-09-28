===description===
Fixing an error in the edited file clears it.
===file:Lib.php===
<?php
function lib(): int { return 1; }
===file:Use.php===
<?php
function run(): string { return lib(); }
===edit:Use.php===
<?php
function run(): int { return lib(); }
===expect===
