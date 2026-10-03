===description===
dirname(__FILE__, 2) does not resolve to the including file's own directory
===config===
file_extensions=module,inc
include_seed=deep/a.module
===file:deep/a.module===
<?php
require dirname(__FILE__, 2) . '/sibling.inc';
function a_hook(): int { return sibling(); }
===file:deep/sibling.inc===
<?php
function sibling(): int { return 1; }
===expect===
a.module: MixedReturnStatement@3:25-3:42: Cannot return a mixed type from function with declared return type 'int'
a.module: UndefinedFunction@3:32-3:41: Function sibling() is not defined
