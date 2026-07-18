===description===
array_key_exists('Iface', class_implements($x)) / array_key_exists('Ancestor',
class_parents($x)) prove $x an instance of that interface/ancestor — the same
relationship `$x instanceof Iface` proves, since both functions return an
array keyed by interface/ancestor-class name. True branch narrows (keeping
subtypes, so a non-final class not itself declaring the interface still
survives as an intersection, same as plain instanceof narrowing); false
branch excludes the exact match. Covers both plain-variable and property
receivers.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
interface Quacks {}
class Animal {}
class Duck extends Animal implements Quacks {}
class Cat extends Animal {}
class Rock {}

/** @param Animal|Duck $obj */
function test_class_implements_true(mixed $obj): void {
    if (array_key_exists('Quacks', class_implements($obj))) {
        /** @mir-check $obj is Animal&Quacks|Duck */
        $_ = $obj;
    }
}

/** @param Duck|Cat $obj */
function test_class_implements_false(mixed $obj): void {
    if (!array_key_exists('Quacks', class_implements($obj))) {
        /** @mir-check $obj is Cat */
        $_ = $obj;
    }
}

/** @param Duck|Rock $obj */
function test_class_parents_true(mixed $obj): void {
    if (array_key_exists('Animal', class_parents($obj))) {
        /** @mir-check $obj is Duck */
        $_ = $obj;
    }
}

class HasAnimalProp {
    /** @var Animal|Duck */
    public mixed $pet;

    public function testPropClassImplements(): void {
        if (array_key_exists('Quacks', class_implements($this->pet))) {
            /** @mir-check $this->pet is Animal&Quacks|Duck */
            $_ = $this->pet;
        }
    }
}
===expect===
PossiblyInvalidArgument@10:35-10:57: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
PossiblyInvalidArgument@18:36-18:58: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
PossiblyInvalidArgument@26:35-26:54: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
MissingConstructor@32:0-32:21: Class HasAnimalProp has uninitialized properties but no constructor
PossiblyInvalidArgument@37:39-37:67: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
