===description===
A chain of includes across four different extensions is followed to its end
===config===
file_extensions=module,inc,install,theme
include_seed=a.module
===file:a.module===
<?php
require_once __DIR__ . '/b.inc';
function a_hook(): int { return d(); }
===file:b.inc===
<?php
require_once __DIR__ . '/c.install';
===file:c.install===
<?php
require_once __DIR__ . '/d.theme';
===file:d.theme===
<?php
function d(): int { return 4; }
===expect===
