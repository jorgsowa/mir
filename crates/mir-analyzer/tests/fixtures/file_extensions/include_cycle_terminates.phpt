===description===
Files that include each other are each visited once
===config===
file_extensions=module,inc
include_seed=a.module
===file:a.module===
<?php
require_once __DIR__ . '/b.inc';
function a_fn(): int { return b_fn(); }
===file:b.inc===
<?php
require_once __DIR__ . '/a.module';
function b_fn(): int { return 1; }
===expect===
