===description===
A callee without a declared return type is re-inferred after its body changes.
===config===
suppress=MissingReturnType
===file:Lib.php===
<?php
function lib() { return 1; }
===file:Use.php===
<?php
function run(): int { return lib(); }
===expect===
===edit:Lib.php===
<?php
function lib() { return 'x'; }
===expect===
Use.php: InvalidReturnType@2:22-2:35: Return type '"x"' is not compatible with declared 'int'
