===description===
Configured extensions are walked: issues surface in .module/.inc files and cross-file calls resolve
===config===
file_extensions=php,module,inc
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
a.module: InvalidReturnType@2:25-2:36: Return type '"x"' is not compatible with declared 'int'
b.inc: InvalidReturnType@3:24-3:35: Return type '"x"' is not compatible with declared 'int'
