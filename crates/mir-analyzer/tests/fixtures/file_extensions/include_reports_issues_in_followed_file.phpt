===description===
A followed .inc file is analyzed, so its own issues are reported under its name
===config===
file_extensions=module,inc
include_seed=a.module
===file:a.module===
<?php
require_once __DIR__ . '/b.inc';
function a_fn(): int { return b_fn(); }
===file:b.inc===
<?php
function b_fn(): int { return 'x'; }
===expect===
b.inc: InvalidReturnType@2:23-2:34: Return type '"x"' is not compatible with declared 'int'
