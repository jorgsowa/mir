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
//                         ^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
//                                ^^^^^^^^^^ UndefinedFunction: Function b_helper() is not defined
