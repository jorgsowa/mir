===description===
A class declared in an included .inc file resolves with its member types
===config===
file_extensions=module,inc
include_seed=a.module
===file:a.module===
<?php
require_once __DIR__ . '/Helper.inc';
function a_fn(): int {
    $h = new Helper();
    return $h->name();
}
===file:Helper.inc===
<?php
class Helper {
    public function name(): string { return 'h'; }
}
===expect===
a.module: InvalidReturnType@5:4-5:22: Return type 'string' is not compatible with declared 'int'
