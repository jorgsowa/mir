===description===
A target whose extension is not configured breaks the chain even when earlier links are followed
===config===
file_extensions=module,inc
include_seed=a.module
===file:a.module===
<?php
require_once __DIR__ . '/b.inc';
function a_hook(): int { return t(); }
===file:b.inc===
<?php
require_once __DIR__ . '/c.tpl';
===file:c.tpl===
<?php
function t(): int { return 1; }
===expect===
a.module: MixedReturnStatement@3:25-3:36: Cannot return a mixed type from function with declared return type 'int'
a.module: UndefinedFunction@3:32-3:35: Function t() is not defined
