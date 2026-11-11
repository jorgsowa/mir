===description===
A string literal that cannot spell a class name is not a `class-string`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

final class Order {}

/** @param class-string $class */
function load($class): void {}

load(Order::class);
load('App\Order');
load('\App\Order');
load('not a class name');
//   ^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $class of load() expects 'class-string', got '"not a class name"'
load('App\Order[]');
//   ^^^^^^^^^^^^^ InvalidArgument: Argument $class of load() expects 'class-string', got '"App\Order[]"'
load('int|null');
//   ^^^^^^^^^^ InvalidArgument: Argument $class of load() expects 'class-string', got '"int|null"'
load('');
//   ^^ InvalidArgument: Argument $class of load() expects 'class-string', got '""'
load('9Lives');
//   ^^^^^^^^ InvalidArgument: Argument $class of load() expects 'class-string', got '"9Lives"'
