===description===
Default extensions walk only .php: .module/.inc files are neither analyzed nor indexed
===file:a.module===
<?php
function a_hook(): int { return 'x'; }
===file:b.inc===
<?php
function b_helper(): string { return 'b'; }
function b_bad(): int { return 'x'; }
===file:c.php===
<?php
function c_use(): string { return b_helper(); }
===expect===
c.php: MixedReturnStatement@2:27-2:45: Cannot return a mixed type from function with declared return type 'string'
c.php: UndefinedFunction@2:34-2:44: Function b_helper() is not defined
