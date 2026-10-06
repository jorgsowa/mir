===description===
An `int|string` (or nullable) argument passed to a refined `positive-int` / `non-empty-string`
param may fail at runtime: ArgumentTypeCoercion (Info), not InvalidArgument. Unions that
contain an unrelated atomic, or whose refined member can never match, stay errors.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param positive-int $id */
function by_id(int $id): void { echo $id; }
/** @param non-empty-string $name */
function by_name(string $name): void { echo $name; }
/** @param positive-int|null $id */
function by_nullable_id(?int $id): void { echo $id; }

function run(int|string $v, ?int $n, ?string $s, int|bool $b, string $str, int|string|float $f): void {
    /** @mir-check $v is int|string */
    by_id($v);
//        ^^ ArgumentTypeCoercion: Argument $id of by_id() expects 'positive-int', got 'int|string' — coercion may fail at runtime
    by_name($v);
//          ^^ ArgumentTypeCoercion: Argument $name of by_name() expects 'non-empty-string', got 'int|string' — coercion may fail at runtime
    by_id($n);
//        ^^ PossiblyNullArgument: Argument $id of by_id() might be null
//        ^^ ArgumentTypeCoercion: Argument $id of by_id() expects 'positive-int', got 'int|null' — coercion may fail at runtime
    by_name($s);
//          ^^ PossiblyNullArgument: Argument $name of by_name() might be null
//          ^^ ArgumentTypeCoercion: Argument $name of by_name() expects 'non-empty-string', got 'string|null' — coercion may fail at runtime
    by_nullable_id($v);
//                 ^^ ArgumentTypeCoercion: Argument $id of by_nullable_id() expects 'positive-int|null', got 'int|string' — coercion may fail at runtime
    by_id($b);
//        ^^ InvalidArgument: Argument $id of by_id() expects 'positive-int', got 'int|bool'
    by_id($str);
//        ^^^^ InvalidArgument: Argument $id of by_id() expects 'positive-int', got 'string'
    by_name($f);
//          ^^ InvalidArgument: Argument $name of by_name() expects 'non-empty-string', got 'int|string|float'
}
