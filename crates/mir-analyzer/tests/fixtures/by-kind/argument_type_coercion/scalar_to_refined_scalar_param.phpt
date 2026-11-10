===description===
A plain `string` or `int` passed to a `non-empty-string` / `positive-int` param may fail at
runtime: ArgumentTypeCoercion. A value already narrowed to the refinement is accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-string $s */
function takes_non_empty(string $s): void {}
/** @param positive-int $i */
function takes_positive(int $i): void {}
/** @param non-empty-string|null $s */
function takes_nullable_non_empty(?string $s): void {}

/** @assert-if-true non-empty-string $s */
function is_non_empty(string $s): bool
{
    return $s !== '';
}

function run(string $s, int $i, ?string $n): void {
    takes_non_empty($s);
//                  ^^ ArgumentTypeCoercion: Argument $s of takes_non_empty() expects 'non-empty-string', got 'string' — coercion may fail at runtime
    takes_positive($i);
//                 ^^ ArgumentTypeCoercion: Argument $i of takes_positive() expects 'positive-int', got 'int' — coercion may fail at runtime
    takes_nullable_non_empty($n);
//                           ^^ ArgumentTypeCoercion: Argument $s of takes_nullable_non_empty() expects 'non-empty-string|null', got 'string|null' — coercion may fail at runtime
    if (is_non_empty($s)) {
        takes_non_empty($s);
    }
    if ($i > 0) {
        takes_positive($i);
    }
}
