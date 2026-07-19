===description===
Loose `==`/`!=` never narrowed an enum-case (`Status::Active`) or a
`get_class()`/`get_debug_type()`/`get_parent_class()`/`$obj::class`
comparison against `Foo::class` — only strict `===`/`!==` had these arms.
Sound to reuse for loose comparison here (enum cases are singleton objects,
and these functions always return plain strings), unlike a general
bare-variable-vs-class-string-literal comparison, which stays unhandled.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
enum Status {
    case Active;
    case Inactive;
}

function narrowsLooseEqualEnumCase(Status $x): void {
    if ($x == Status::Active) {
        /** @mir-check $x is Status::Active */
        $_ = 1;
    }
}

function narrowsLooseNotEqualEnumCase(Status $x): void {
    if ($x != Status::Active) {
        /** @mir-check $x is Status::Inactive */
        $_ = 1;
    }
}

class Animal {}
class Dog extends Animal {}
class Cat extends Animal {}

function narrowsLooseGetClassAgainstClassConst(Animal $a): void {
    if (get_class($a) == Dog::class) {
        /** @mir-check $a is Dog */
        $_ = 1;
    }
}

function narrowsLooseGetDebugTypeAgainstClassConst(Animal $a): void {
    if (get_debug_type($a) == Dog::class) {
        /** @mir-check $a is Dog */
        $_ = 1;
    }
}

final class Holder {
    public Animal $pet;

    public function narrowsLoosePropGetClass(): void {
        if (get_class($this->pet) == Dog::class) {
            /** @mir-check $this->pet is Dog */
            $_ = 1;
        }
    }
}
===expect===
