===description===
`@psalm-assert array<int, string> $value` narrows a mixed argument to the generic array type:
its elements read as strings and an `is_int()` check on the array is always false. A type
check on a generic array's element is not reported, as for a declared array type.
===file===
<?php
declare(strict_types=1);

/** @psalm-assert array<int, string> $value */
function assertIntStringMap(mixed $value): void {
    if (!\is_array($value)) {
        throw new \InvalidArgumentException('x');
    }
}

/** @psalm-assert list<int> $value */
function assertIntList(mixed $value): void {
    if (!\is_array($value)) {
        throw new \InvalidArgumentException('x');
    }
}

/** @psalm-assert array $value */
function assertArray(mixed $value): void {
    if (!\is_array($value)) {
        throw new \InvalidArgumentException('x');
    }
}

function takesString(string $value): void {
    echo $value;
}

function example(mixed $value): void {
    assertIntStringMap($value);
    takesString($value[0]);
    echo \is_int($value[0]) ? 'int' : 'string';
}

function unasserted(mixed $value): void {
    takesString($value[0]);
//              ^^^^^^^^^ MixedArgument: Argument $value of takesString() is mixed
//              ^^^^^^^^^ MixedArrayAccess: Array access on mixed type
}

function listExample(mixed $value): void {
    assertIntList($value);
    /** @mir-check $value is list<int> */
    echo \count($value);
}

function plainArrayExample(mixed $value): void {
    assertArray($value);
    echo \is_int($value[0]) ? 'int' : 'string';
}

/** @param array<int, string> $value */
function declared(array $value): void {
    echo \is_int($value[0]) ? 'int' : 'string';
    echo \is_int($value) ? 'int' : 'string';
//       ^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}

function scalarAssert(mixed $value): void {
    assertIntStringMap($value);
    /** @mir-check $value is array<int, string> */
    echo \is_int($value) ? 'int' : 'string';
//       ^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}
