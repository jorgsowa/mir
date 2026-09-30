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
//      ^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $bar on possibly null value
        $foo->ping();
    }
}

function viaBoolTrue(?Foo $foo): void {
    if ($foo->flag === true) {
//      ^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $flag on possibly null value
        $foo->ping();
    }
}

function viaInt(?Foo $foo): void {
    if ($foo->num === 42) {
//      ^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $num on possibly null value
        $foo->ping();
    }
}

function viaString(?Foo $foo): void {
    if ($foo->label === 'x') {
//      ^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $label on possibly null value
        $foo->ping();
    }
}

function viaEnumCase(?Foo $foo): void {
    if ($foo->status === Status::Active) {
//      ^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $status on possibly null value
        $foo->ping();
    }
}

// Negative: the excluded branch proves nothing about $foo itself.
function viaInstanceofFalseBranch(?Foo $foo): void {
    if (!($foo->bar instanceof Baz)) {
//        ^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $bar on possibly null value
        $foo->ping();
//      ^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method ping() on possibly null value
    }
}
===expect===
