===description===
A negated assertion on a scalar type (`@psalm-assert !int`, `@phpstan-assert !string`) subtracts it from the argument's type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @psalm-assert !int $v */
function assertNotInt(mixed $v): void {}

/** @phpstan-assert !string $v */
function assertNotString(mixed $v): void {}

/** @phpstan-assert !array $v */
function assertNotArray(mixed $v): void {}

function takesString(string $s): void {}

function ints(int|string $v): void {
    assertNotInt($v);
    takesString($v);
    /** @mir-check $v is string */
    echo 1;
}

function strings(int|float|string $v): void {
    assertNotString($v);
    /** @mir-check $v is int|float */
    echo 1;
}

/** @param array<int>|bool $v */
function arrays(array|bool $v): void {
    assertNotArray($v);
    /** @mir-check $v is bool */
    echo 1;
}

/** @param positive-int|string $v */
function refined(int|string $v): void {
    assertNotInt($v);
    /** @mir-check $v is string */
    echo 1;
}
