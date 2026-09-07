===description===
`get_parent_class(self::$prop) === Foo::class` (and its symmetric,
string-literal, and loose-`==` variants) narrows a static property to a
strict subclass — `ScalarArgTarget` has no static-property variant
(tracked as S19), so these previously matched neither Var nor Prop on a
static receiver and narrowed nothing.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
class Animal {}
class Dog extends Animal {}
class Puppy extends Dog {}
class Cat extends Animal {}

class Box {
    /** @var Dog|Puppy|Cat */
    public static Animal $pet;
}

function classConstEq(): void {
    if (get_parent_class(Box::$pet) === Dog::class) {
        /** @mir-check Box::$pet is Puppy */
        $_ = 1;
    }
}

function classConstEqSymmetric(): void {
    if (Dog::class === get_parent_class(Box::$pet)) {
        /** @mir-check Box::$pet is Puppy */
        $_ = 1;
    }
}

function stringLiteralEq(): void {
    if (get_parent_class(Box::$pet) === 'Dog') {
        /** @mir-check Box::$pet is Puppy */
        $_ = 1;
    }
}

function stringLiteralEqSymmetric(): void {
    if ('Dog' === get_parent_class(Box::$pet)) {
        /** @mir-check Box::$pet is Puppy */
        $_ = 1;
    }
}

function looseEq(): void {
    if (get_parent_class(Box::$pet) == 'Dog') {
        /** @mir-check Box::$pet is Puppy */
        $_ = 1;
    }
}

function looseEqSymmetric(): void {
    if ('Dog' == get_parent_class(Box::$pet)) {
        /** @mir-check Box::$pet is Puppy */
        $_ = 1;
    }
}
===expect===
