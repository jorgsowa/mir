===description===
M28: class_implements()/class_parents()/class_uses() genuinely can't return
false once a preceding class_exists()/interface_exists() guard already proved
the argument names a loaded class — the stub's bare `array|false` return only
models the "class doesn't exist" case. Covers both a guarded variable
receiver and a guarded identical-literal receiver; a negative control (no
guard) confirms the fix is scoped, not blanket.
===config===
suppress=UnusedParam
===file===
<?php
interface Iterator2 {}
class Guarded implements Iterator2 {}

function guardedVariable(string $className): void {
    if (!class_exists($className)) {
        return;
    }
    if (!in_array(Iterator2::class, class_implements($className), true)) {
        return;
    }
}

function guardedLiteral(): void {
    if (!class_exists(Guarded::class)) {
        return;
    }
    if (!in_array('Iterator2', class_implements(Guarded::class), true)) {
        return;
    }
}

function guardedParents(string $className): void {
    if (!class_exists($className)) {
        return;
    }
    if (!in_array('Guarded', class_parents($className), true)) {
        return;
    }
}

function guardedUses(string $className): void {
    if (!class_exists($className)) {
        return;
    }
    if (!in_array('SomeTrait', class_uses($className), true)) {
        return;
    }
}

function unguarded(string $className): void {
    if (!in_array(Iterator2::class, class_implements($className), true)) {
        return;
    }
}
===expect===
PossiblyInvalidArgument@42:36-42:64: Argument $haystack of in_array() expects 'array', possibly different type 'array<int|string, string>|false' provided
