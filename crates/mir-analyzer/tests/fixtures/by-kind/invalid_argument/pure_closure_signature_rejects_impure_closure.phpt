===description===
`pure-Closure(int): int` and `pure-callable(int): int` reject an impure closure; a parameter that also takes a plain callable accepts it.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param pure-Closure(int): int $step */
function iterate(\Closure $step): void {}

/** @param pure-callable(int): int $step */
function fold(callable $step): void {}

/** @param pure-callable(int): int|callable(int): int $step */
function either(callable $step): void {}

$traced = static function (int $n): int {
    echo 'step';
    return $n + 1;
};
$quiet = static fn (int $n): int => $n + 1;

iterate($quiet);
iterate($traced);
//      ^^^^^^^ InvalidArgument: Argument $step of iterate() expects 'pure-Closure(int): int', got 'impure Closure(int): int'
fold($quiet);
fold($traced);
//   ^^^^^^^ InvalidArgument: Argument $step of fold() expects 'pure-callable(int): int', got 'impure Closure(int): int'
either($traced);
