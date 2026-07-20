===description===
The three OR-disjunct property narrowers (pure instanceof, pure type-fn,
mixed) now propagate receiver-non-null on their property receiver, like
every single-disjunct property narrowing already does. An `is_null(...)`
disjunct is excluded since it doesn't prove non-null (a null receiver's
->prop read is itself null).
===config===
suppress=UnusedVariable,PossiblyNullPropertyFetch,MissingPropertyType
===file===
<?php
interface Foo {}
interface Bar {}

class Box {
    /** @var Foo|Bar|null */
    public $value;
}

function instanceofDisjunctsProveReceiverNonNull(?Box $x): void {
    if ($x->value instanceof Foo || $x->value instanceof Bar) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function typeFnDisjunctsProveReceiverNonNull(?Box $x): void {
    if (is_int($x->value) || is_string($x->value)) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function typeFnDisjunctsWithIsNullDoNotProveReceiverNonNull(?Box $x): void {
    if (is_null($x->value) || is_string($x->value)) {
        /** @mir-check $x is ?Box */
        $_ = 1;
    }
}

function mixedDisjunctsProveReceiverNonNull(?Box $x): void {
    if ($x->value instanceof Foo || is_string($x->value)) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function mixedDisjunctsWithIsNullDoNotProveReceiverNonNull(?Box $x): void {
    if (is_null($x->value) || $x->value instanceof Foo) {
        /** @mir-check $x is ?Box */
        $_ = 1;
    }
}

===expect===
