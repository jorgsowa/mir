===description===
`non-empty-scalar`, `non-empty-mixed` and `non-null-mixed` reject the values they exclude: falsy scalars, `[]` and `null`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param non-empty-scalar $value */
function nonEmptyScalar($value): void {}

/** @param non-empty-mixed $value */
function nonEmptyMixed($value): void {}

/** @param non-null-mixed $value */
function nonNullMixed($value): void {}

nonEmptyScalar(1);
nonEmptyScalar(1.5);
nonEmptyScalar('x');
nonEmptyScalar(true);
nonEmptyScalar('');
//             ^^ InvalidArgument: Argument $value of nonEmptyScalar() expects 'non-falsy-string|negative-int|positive-int|float|true', got '""'
nonEmptyScalar('0');
//             ^^^ InvalidArgument: Argument $value of nonEmptyScalar() expects 'non-falsy-string|negative-int|positive-int|float|true', got '"0"'
nonEmptyScalar(false);
//             ^^^^^ InvalidArgument: Argument $value of nonEmptyScalar() expects 'non-falsy-string|negative-int|positive-int|float|true', got 'false'

nonEmptyMixed('x');
nonEmptyMixed([1]);
nonEmptyMixed(new stdClass());
nonEmptyMixed(null);
//            ^^^^ NullArgument: Argument $value of nonEmptyMixed() cannot be null

nonNullMixed(0);
nonNullMixed('');
nonNullMixed([]);
nonNullMixed(new stdClass());
nonNullMixed(null);
//           ^^^^ NullArgument: Argument $value of nonNullMixed() cannot be null
