===description===
require, require_once, include, dirname(__FILE__), bare relative literal and dotted __DIR__ paths all resolve
===config===
file_extensions=module,inc,install,theme,profile
include_seed=a.module
===file:a.module===
<?php
require __DIR__ . '/r1.inc';
require_once __DIR__ . '/r2.install';
include __DIR__ . '/r3.theme';
include_once dirname(__FILE__) . '/r4.profile';
require_once 'sub/r5.inc';
require __DIR__ . '/sub/../r6.inc';
function a_hook(): int { return r1() + r2() + r3() + r4() + r5() + r6(); }
===file:r1.inc===
<?php
function r1(): int { return 1; }
===file:r2.install===
<?php
function r2(): int { return 2; }
===file:r3.theme===
<?php
function r3(): int { return 3; }
===file:r4.profile===
<?php
function r4(): int { return 4; }
===file:sub/r5.inc===
<?php
function r5(): int { return 5; }
===file:r6.inc===
<?php
function r6(): int { return 6; }
===expect===
