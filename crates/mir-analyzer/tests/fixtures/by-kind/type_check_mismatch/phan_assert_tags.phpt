===description===
`@phan-assert`, `@phan-assert-true-condition` and `@phan-assert-false-condition` narrow like their `@psalm-assert` counterparts.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @phan-assert int $value */
function assertIsInt(mixed $value): void {}

/** @phan-assert-true-condition string $value */
function isString(mixed $value): bool { return is_string($value); }

/** @phan-assert-false-condition int $value */
function isNotInt(mixed $value): bool { return !is_int($value); }

function asserted(mixed $value): void {
    assertIsInt($value);
    /** @mir-check $value is int */
    echo 1;
}

function trueCondition(mixed $value): void {
    if (isString($value)) {
        /** @mir-check $value is string */
        echo 1;
    }
}

function falseCondition(mixed $value): void {
    if (!isNotInt($value)) {
        /** @mir-check $value is int */
        echo 1;
    }
}
