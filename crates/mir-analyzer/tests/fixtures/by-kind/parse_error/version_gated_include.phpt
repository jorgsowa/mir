===description===
A file every analyzed `require`/`include` of loads only under a `PHP_VERSION_ID` check above the target version reports no version-too-low errors, as PHP never compiles it there; a file loaded unguarded, or under a lower check, still does.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file:bootstrap.php===
<?php
if (\PHP_VERSION_ID >= 80500) {
    require __DIR__.'/bootstrap85.php';
}
if (PHP_VERSION_ID < 80500) {
    define('LEGACY', true);
} else {
    require __DIR__ . '/else85.php';
}
if (80400 < PHP_VERSION_ID && extension_loaded('uuid')) {
    require_once __DIR__.'/above84.php';
}
if (PHP_VERSION_ID >= 80400) {
    require __DIR__.'/from84.php';
}
if (PHP_VERSION_ID >= 80500) {
    require __DIR__.'/both.php';
}
require __DIR__.'/both.php';
if (PHP_VERSION_ID < 80000) {
    define('PHP7', true);
} elseif (PHP_VERSION_ID < 80400) {
    define('PHP8', true);
} elseif (PHP_VERSION_ID > 80499) {
    require __DIR__.'/elseif85.php';
}
===file:bootstrap85.php===
<?php
#[Deprecated(message: 'use B instead')]
const A = 1;
===file:else85.php===
<?php
#[Deprecated]
const B = 1;
===file:above84.php===
<?php
#[Deprecated]
const C = 1;
===file:from84.php===
<?php
#[Deprecated]
const D = 1;
//<^^^^^ ParseError: 'attributes on constants' requires PHP 8.5 or higher [version-too-low]
===file:both.php===
<?php
#[Deprecated]
const E = 1;
//<^^^^^ ParseError: 'attributes on constants' requires PHP 8.5 or higher [version-too-low]
===file:elseif85.php===
<?php
#[Deprecated]
const G = 1;
===file:orphan85.php===
<?php
#[Deprecated]
const F = 1;
//<^^^^^ ParseError: 'attributes on constants' requires PHP 8.5 or higher [version-too-low]
