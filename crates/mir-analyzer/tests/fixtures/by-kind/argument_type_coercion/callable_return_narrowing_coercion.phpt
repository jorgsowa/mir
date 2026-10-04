===description===
Under strict_types=1, a closure returning a broader type than the documented callable return
(int vs positive-int or -1|0|1) may fail at runtime: ArgumentTypeCoercion (Info). Unrelated
or nullable return types stay InvalidArgument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param Closure(): positive-int $c */
function takes_positive(Closure $c): void { $c(); }
/** @param callable(string, string):(-1|0|1) $c */
function takes_comparator(callable $c): void { $c('a', 'b'); }

function run(int $i, string $s, ?int $n): void {
    takes_positive(fn(): int => $i);
//                 ^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $c of takes_positive() expects 'callable returning positive-int', got 'callable returning int' — coercion may fail at runtime
    takes_positive(function () use ($i): int { return $i; });
//                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $c of takes_positive() expects 'callable returning positive-int', got 'callable returning int' — coercion may fail at runtime
    takes_comparator(fn(string $a, string $b): int => $i);
//                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $c of takes_comparator() expects 'callable returning -1|0|1', got 'callable returning int' — coercion may fail at runtime
    takes_positive(fn(): string => $s);
//                 ^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $c of takes_positive() expects 'callable returning positive-int', got 'callable returning string'
    takes_positive(fn(): ?int => $n);
//                 ^^^^^^^^^^^^^^^^ InvalidArgument: Argument $c of takes_positive() expects 'callable returning positive-int', got 'callable returning int|null'
}
===expect===
