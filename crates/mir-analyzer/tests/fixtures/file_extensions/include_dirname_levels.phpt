===description===
dirname(__FILE__, N), dirname(__DIR__) and parenthesized concatenation resolve to the right directory
===config===
file_extensions=module,inc
include_seed=deep/er/a.module
===file:deep/er/a.module===
<?php
require dirname(__FILE__, 2) . '/up2.inc';
require dirname(__DIR__) . '/up1.inc';
require (__DIR__ . '/../') . 'paren.inc';
function a_hook(): int { return up2() + up1() + paren(); }
===file:deep/up2.inc===
<?php
function up2(): int { return 2; }
===file:deep/up1.inc===
<?php
function up1(): int { return 1; }
===file:deep/paren.inc===
<?php
function paren(): int { return 3; }
===expect===
