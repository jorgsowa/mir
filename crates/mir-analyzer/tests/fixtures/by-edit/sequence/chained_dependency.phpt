===description===
A signature edit propagates through an unchanged intermediate caller.
===config===
suppress=MissingReturnType
===file:A.php===
<?php
function a(): int { return 1; }
===file:B.php===
<?php
function b() { return a(); }
===file:C.php===
<?php
function c(): int { return b(); }
===expect===
<<none>>
===edit:A.php===
<?php
function a(): string { return ''; }
===expect===
C.php: InvalidReturnType@2:20-2:31: Return type 'string' is not compatible with declared 'int'
