===description===
The `!Type` negated form of an assertion annotation (`@psalm-assert !null
$x`, `@phpstan-assert-if-true !Foo $x`) was not recognized at all: the
docblock parser had no handling for a leading `!`, so `parse_type_string`
parsed the literal text "!null" as an unrelated bare type instead of "$x is
NOT null" — silently producing a useless assertion instead of narrowing.
Also covers a plain (non-negated) `@phpstan-assert` used as a bare
statement, not just inside a condition, which already worked for
one-argument functions via the general call-analysis path but is pinned
here alongside the negated form.
===config===
suppress=MissingPropertyType
===file===
<?php
/**
 * @param mixed $x
 * @phpstan-assert !null $x
 */
function ensureNotNull($x): void {
    if ($x === null) {
        throw new \InvalidArgumentException("must not be null");
    }
}

function greet(?string $x): string {
    ensureNotNull($x);
    return $x;
}

class Animal {}
class Dog extends Animal {}

/**
 * @param mixed $x
 * @psalm-assert !Dog $x
 */
function ensureNotDog($x): void {
    if ($x instanceof Dog) {
        throw new \InvalidArgumentException("must not be a Dog");
    }
}

/** @param Animal $a */
function notADog(Animal $a): Animal {
    ensureNotDog($a);
    return $a;
}

/**
 * @param mixed $x
 * @phpstan-assert-if-true !null $x
 */
function isNotNull($x): bool {
    return $x !== null;
}

function useIfTrue(?string $x): string {
    if (isNotNull($x)) {
        return $x;
    }
    return "default";
}
===expect===
