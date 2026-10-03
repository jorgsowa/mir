===description===
A plain .php file including a .inc file follows it
===config===
file_extensions=php,module,inc
include_seed=index.php
===file:index.php===
<?php
require_once __DIR__ . '/core.inc';
function boot(): string { return core_name(); }
===file:core.inc===
<?php
function core_name(): string { return 'core'; }
===expect===
