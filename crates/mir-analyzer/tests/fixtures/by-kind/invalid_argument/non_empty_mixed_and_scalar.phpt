===description===
`non-empty-mixed`, `non-null-mixed` and `non-empty-scalar` accept truthy values; a non-scalar is still rejected for the scalar form.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/** @param non-empty-mixed $value */
function acceptsNonEmptyMixed($value): void {}

/** @param non-null-mixed $value */
function acceptsNonNullMixed($value): void {}

/** @param non-empty-scalar $value */
function acceptsNonEmptyScalar($value): void {}

acceptsNonEmptyMixed(1);
acceptsNonEmptyMixed('x');
acceptsNonEmptyMixed([1]);
acceptsNonEmptyMixed(new \stdClass());

acceptsNonNullMixed(5);
acceptsNonNullMixed('x');

acceptsNonEmptyScalar(1);
acceptsNonEmptyScalar(1.5);
acceptsNonEmptyScalar('x');
acceptsNonEmptyScalar(true);
acceptsNonEmptyScalar([1]);
//                    ^^^ InvalidArgument: Argument $value of acceptsNonEmptyScalar() expects 'scalar', got 'array{0: 1}'
