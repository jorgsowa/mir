===description===
A variable holding one literal function name is analyzed like a direct call: unknown names,
arity and argument types are checked; guards, `Class::method` strings, unions, and
existing functions stay quiet.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function helper(int $n): int { return $n; }
function fill(array &$out): void { $out = [1]; }
class Tool { public static function run(): void {} }

function unknownName(): void {
    $fn = 'no_such_fn_xyz';
    $fn(1);
//  ^^^^^^ UndefinedFunction: Function no_such_fn_xyz() is not defined
}
function knownArity(): void {
    $fn = 'helper';
    $fn();
//  ^^^^^ TooFewArguments: Too few arguments for helper(): expected 1, got 0
    $fn(1, 2);
//         ^ TooManyArguments: Too many arguments for helper(): expected 1, got 2
}
function knownArgType(): void {
    $fn = '\helper';
    $fn([]);
//      ^^ InvalidArgument: Argument $n of helper() expects 'int', got 'array{}'
}
function returnTypeFlows(): void {
    $fn = 'helper';
    $r = $fn(1);
    /** @mir-check $r is int */
    echo $r;
}
function builtinOk(): void {
    $fn = 'strlen';
    $n = $fn('abc');
    /** @mir-check $n is int */
    echo $n;
}
function byRefWritesBack(): void {
    $fn = 'fill';
    $fn($out);
    /** @mir-check $out is array */
    echo count($out);
}
function guardedByFunctionExists(): void {
    $fn = 'maybe_defined_fn';
    if (function_exists($fn)) {
        $fn();
    }
    if (!function_exists($fn)) {
        return;
    }
    $fn();
}
function guardedByIsCallable(): void {
    $fn = 'maybe_defined_fn';
    if (is_callable($fn)) {
        $fn();
    }
    if (!is_callable($fn)) {
        return;
    }
    $fn();
}
function guardedByLiteralName(): void {
    $fn = 'maybe_defined_fn';
    if (function_exists('maybe_defined_fn')) {
        $fn();
    }
}
function methodStringsAreSkipped(): void {
    $fn = 'Tool::run';
    $fn();
    $other = 'Missing::nope';
    $other();
}
function unionCalleeIsSkipped(bool $b): void {
    $fn = $b ? 'no_such_fn_a' : 'helper';
    $fn(1);
}
function nonIdentifierStringIsSkipped(): void {
    $fn = 'not a function';
    $fn();
}
