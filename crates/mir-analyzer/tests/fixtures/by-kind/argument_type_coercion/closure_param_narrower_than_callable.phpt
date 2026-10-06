===description===
A closure param narrower than the documented callable param (int vs int|string, subclass vs
parent) is an ArgumentTypeCoercion. Disjoint param types stay InvalidArgument, and a param coercion does
not hide an incompatible return.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
class Child extends Base {}

/** @param callable(int|string, string): mixed $f */
function takes_scalar_callback(callable $f): void {}
/** @param callable(Base): mixed $f */
function takes_object_callback(callable $f): void {}
/** @param callable(Base): int $f */
function takes_int_returning(callable $f): void {}

function run(): void {
    takes_scalar_callback(fn(int $k, string $v) => $k);
//                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $f of takes_scalar_callback() expects 'callable whose parameter #1 accepts int|string', got 'callable whose parameter #1 only accepts int' — coercion may fail at runtime
    takes_scalar_callback(fn(int|string $k, string $v) => $k);
    takes_object_callback(fn(Child $c) => 1);
//                        ^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $f of takes_object_callback() expects 'callable whose parameter #1 accepts Base', got 'callable whose parameter #1 only accepts Child' — coercion may fail at runtime
    takes_object_callback(fn(Base $b) => 1);
    takes_object_callback(fn(stdClass $o) => 1);
//                        ^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $f of takes_object_callback() expects 'callable whose parameter #1 accepts Base', got 'callable whose parameter #1 only accepts stdClass'
    takes_scalar_callback(fn(float $k, string $v) => $k);
//                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $f of takes_scalar_callback() expects 'callable whose parameter #1 accepts int|string', got 'callable whose parameter #1 only accepts float'
    takes_int_returning(fn(Child $c): string => 'x');
//                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $f of takes_int_returning() expects 'callable whose parameter #1 accepts Base', got 'callable whose parameter #1 only accepts Child' — coercion may fail at runtime
//                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $f of takes_int_returning() expects 'callable returning int', got 'callable returning non-empty-string'
}
