===description===
Standalone scripts that declare the same function name are each checked against their own declaration; a file without its own declaration still resolves the workspace one.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:first.php===
<?php
function build(int $a): int { return $a; }
function inferred(int $a) { return $a; }

$n = build(1);
/** @mir-check $n is int */
$_ = $n;
$i = inferred(1);
/** @mir-check $i is int */
$_ = $i;
$fn = build(...);
$mapped = array_map('build', [1, 2]);
build('wrong');
===file:second.php===
<?php
function build(string $a, string $b): string { return $a . $b; }
function inferred(string $a) { return $a; }

$s = build('x', 'y');
/** @mir-check $s is string */
$_ = $s;
$i = inferred('x');
/** @mir-check $i is string */
$_ = $i;
$fn = build(...);
$mapped = array_map('build', ['a'], ['b']);
build(1, 2);
===file:third.php===
<?php
$_ = unique_helper(1);
function unique_helper(int $a): int { return $a; }
===file:fourth.php===
<?php
$_ = unique_helper(2);
===expect===
first.php: InvalidArgument@13:6-13:13: Argument $a of build() expects 'int', got '"wrong"'
second.php: ArgumentTypeCoercion@13:6-13:7: Argument $a of build() expects 'string', got '1' — coercion may fail at runtime
second.php: ArgumentTypeCoercion@13:9-13:10: Argument $b of build() expects 'string', got '2' — coercion may fail at runtime
