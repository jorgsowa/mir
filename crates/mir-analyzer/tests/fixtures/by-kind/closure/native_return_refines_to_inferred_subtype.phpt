===description===
Under strict types, a closure's native `int`/`string` return is refined by a body that provably returns a narrower int/string class, so it satisfies a narrower declared callable return. Literals widen to their sign / non-emptiness class; non-subtype and non-family bodies keep the declared type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnnecessaryVarAnnotation errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param \Closure(): positive-int $f */
function takesPositive(\Closure $f): void {}

/** @param callable(): non-empty-string $f */
function takesNonEmpty(callable $f): void {}

/** @param \Closure(): non-negative-int $f */
function takesNonNegative(\Closure $f): void {}

final class Limits {
    public const SIZE = 5;
}

function accepted(int $n, string $s): void {
    takesPositive(fn(): int => 5);
    takesPositive(fn(): int => Limits::SIZE);
    takesPositive(fn(): int => $n > 0 ? $n : 1);
    takesPositive(static function (): int { return 7; });
    takesNonEmpty(fn(): string => 'x');
    takesNonEmpty(fn(): string => 'p' . $s);
    takesNonNegative(fn(): int => abs($n));
    takesNonNegative(fn(): int => 0);
}

function rejected(int $n, string $s, ?int $maybe): void {
    takesPositive(fn(): int => $n);
    takesPositive(fn(): int => 0);
    takesPositive(fn(): int => -3);
    takesNonEmpty(fn(): string => $s);
    takesNonEmpty(fn(): string => '');
    takesPositive(fn(): int => $n > 0 ? 1 : -1);
}

function recorded(int $n, string $s, ?int $maybe, mixed $m): void {
    $literal = fn(): int => 5;
    /** @mir-check $literal is Closure(): positive-int */
    $negative = fn(): int => -3;
    /** @mir-check $negative is Closure(): negative-int */
    $range = fn(): int => $n > 0 ? $n : 1;
    /** @mir-check $range is Closure(): int<1, max> */
    $signs = fn(): int => $n > 0 ? 1 : -1;
    /** @mir-check $signs is Closure(): int */
    $positives = fn(): int => $n > 0 ? 1 : 2;
    /** @mir-check $positives is Closure(): positive-int */
    $text = fn(): string => 'x';
    /** @mir-check $text is Closure(): non-empty-string */
    $plain = fn(): int => $n;
    /** @mir-check $plain is Closure(): int */
    $nullable = fn(): ?int => $maybe;
    /** @mir-check $nullable is Closure(): int|null */
    $untyped = fn(): int => (int) $m;
    /** @mir-check $untyped is Closure(): int */
    $open = fn() => 5;
    /** @mir-check $open is Closure(): 5 */
}
===expect===
InvalidArgument@29:18-29:33: Argument $f of takesPositive() expects 'callable returning positive-int', got 'callable returning int'
InvalidArgument@30:18-30:32: Argument $f of takesPositive() expects 'callable returning positive-int', got 'callable returning non-negative-int'
InvalidArgument@31:18-31:33: Argument $f of takesPositive() expects 'callable returning positive-int', got 'callable returning negative-int'
InvalidArgument@32:18-32:36: Argument $f of takesNonEmpty() expects 'callable returning non-empty-string', got 'callable returning string'
InvalidArgument@33:18-33:36: Argument $f of takesNonEmpty() expects 'callable returning non-empty-string', got 'callable returning string'
InvalidArgument@34:18-34:46: Argument $f of takesPositive() expects 'callable returning positive-int', got 'callable returning int'
