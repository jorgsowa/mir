===description===
After `@phpstan-assert !int` on an `int|string` value, an `is_int()` check on it is always false.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @phpstan-assert !int $value */
function assertNotInt(int|string $value): void {
    if (is_int($value)) {
        throw new InvalidArgumentException('must not be int');
    }
}

function example(int|string $value): string {
    assertNotInt($value);
    return is_int($value) ? 'int' : 'string';
    //     ^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}
