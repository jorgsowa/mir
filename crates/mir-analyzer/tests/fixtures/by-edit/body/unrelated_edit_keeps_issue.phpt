===description===
An edit that leaves the callee's signature intact keeps the caller's existing issue.
===file:Lib.php===
<?php
function lib(): string { return ''; }
===file:Use.php===
<?php
function run(): int { return lib(); }
===edit:Lib.php===
<?php
function lib(): string { return 'changed'; }
function added(): void {}
===expect===
Use.php: InvalidReturnType@2:22-2:35: Return type 'string' is not compatible with declared 'int'
