===description===
Proving `$obj->prop` equals a definite non-null value (instanceof, bool,
int, string, or enum-case literal match) must also prove `$obj` itself is
non-null — PHP 8 reads `$obj->prop` on a null `$obj` as a warning, still
evaluating to null, same ambiguity as the already-fixed nullsafe/null-check
case. Only the matched-true direction proves this; the excluded/false
direction (last function) proves nothing about the receiver.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
enum Status {
    case Active;
    case Inactive;
}

class Bar {}
class Baz extends Bar {}

class Foo {
    public Bar $bar;
    public bool $flag = false;
    public int $num = 0;
    public string $label = '';
    public Status $status = Status::Active;

    public function ping(): void {}
}

function viaInstanceof(?Foo $foo): void {
    if ($foo->bar instanceof Baz) {
        $foo->ping();
    }
}

function viaBoolTrue(?Foo $foo): void {
    if ($foo->flag === true) {
        $foo->ping();
    }
}

function viaInt(?Foo $foo): void {
    if ($foo->num === 42) {
        $foo->ping();
    }
}

function viaString(?Foo $foo): void {
    if ($foo->label === 'x') {
        $foo->ping();
    }
}

function viaEnumCase(?Foo $foo): void {
    if ($foo->status === Status::Active) {
        $foo->ping();
    }
}

// Negative: the excluded branch proves nothing about $foo itself.
function viaInstanceofFalseBranch(?Foo $foo): void {
    if (!($foo->bar instanceof Baz)) {
        $foo->ping();
    }
}
===expect===
PossiblyNullPropertyFetch@21:8-21:17: Cannot access property $bar on possibly null value
PossiblyNullPropertyFetch@27:8-27:18: Cannot access property $flag on possibly null value
PossiblyNullPropertyFetch@33:8-33:17: Cannot access property $num on possibly null value
PossiblyNullPropertyFetch@39:8-39:19: Cannot access property $label on possibly null value
PossiblyNullPropertyFetch@45:8-45:20: Cannot access property $status on possibly null value
PossiblyNullPropertyFetch@52:10-52:19: Cannot access property $bar on possibly null value
PossiblyNullMethodCall@53:8-53:20: Cannot call method ping() on possibly null value
