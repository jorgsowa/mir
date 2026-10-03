===description===
A bare relative literal resolves against the including file's directory, not the project root
===config===
file_extensions=module,inc
include_seed=sub/a.module
===file:sub/a.module===
<?php
require_once 'b.inc';
function a_fn(): int { return sub_fn(); }
===file:sub/b.inc===
<?php
function sub_fn(): int { return 1; }
===file:b.inc===
<?php
function sub_fn(): string { return 'root'; }
===expect===
