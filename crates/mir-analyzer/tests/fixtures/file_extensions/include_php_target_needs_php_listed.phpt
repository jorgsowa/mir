===description===
With php absent from the list, an include of a .php file is not followed
===config===
file_extensions=module,inc
include_seed=a.module
===file:a.module===
<?php
require_once __DIR__ . '/b.php';
function a_fn(): int { return b_fn(); }
===file:b.php===
<?php
function b_fn(): int { return 1; }
===expect===
a.module: MixedReturnStatement@3:23-3:37: Cannot return a mixed type from function with declared return type 'int'
a.module: UndefinedFunction@3:30-3:36: Function b_fn() is not defined
