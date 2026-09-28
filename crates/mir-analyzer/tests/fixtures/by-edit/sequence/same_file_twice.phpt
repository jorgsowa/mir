===description===
Only the last of several edits to one file determines the result.
===file:Lib.php===
<?php
function lib(): int { return 1; }
===file:Use.php===
<?php
function run(): int { return lib(); }
===expect===
===edit:Lib.php===
<?php
function lib(): string { return ''; }
===edit:Lib.php===
<?php
function lib(): int { return 2; }
===expect===
