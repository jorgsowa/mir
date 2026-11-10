===description===
A ternary check contradicting a type established by an assertion function is reported, for plain, conditional and templated assertions.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @phpstan-assert int $value */
function assertIsInt(mixed $value): void {
    if (!is_int($value)) {
        throw new \InvalidArgumentException('not an int');
    }
}

/** @psalm-assert-if-true int $value */
function isInt(mixed $value): bool {
    return is_int($value);
}

/**
 * @template T
 * @param T $expected
 * @psalm-assert T $actual
 */
function assertSame(mixed $expected, mixed $actual): void {
    if ($expected !== $actual) {
        throw new \InvalidArgumentException('not the same');
    }
}

function afterAssert(mixed $value): string {
    assertIsInt($value);
    return is_string($value) ? 'string' : 'int';
//         ^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}

function insideAssertIfTrue(mixed $value): string {
    if (isInt($value)) {
        return is_string($value) ? 'string' : 'int';
//             ^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
    }
    return is_string($value) ? 'string' : 'other';
}

function afterTemplatedAssert(mixed $value): string {
    assertSame('x', $value);
    return is_int($value) ? 'int' : 'string';
//         ^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}

function withoutAssert(mixed $value): string {
    return is_string($value) ? 'string' : 'other';
}
