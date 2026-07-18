===description===
get_parent_class($x) === 'ClassName' (or === ClassName::class) proves $x's
class's immediate parent is exactly ClassName, so $x is a strict subclass
instance of ClassName — the same relationship is_subclass_of() proves, and
narrowed the same way (true branch narrows, false branch stays unchanged).
Covers both plain-variable and property receivers.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Animal {}
class Dog extends Animal {}
class Cat extends Animal {}

/** @param Animal|Dog $obj */
function test_var_string_literal(mixed $obj): void {
    if (get_parent_class($obj) === 'Animal') {
        /** @mir-check $obj is Dog */
        $_ = $obj;
    }
}

/** @param Animal|Dog $obj */
function test_var_class_const_reversed(mixed $obj): void {
    if (Animal::class === get_parent_class($obj)) {
        /** @mir-check $obj is Dog */
        $_ = $obj;
    }
}

/** @param Animal $obj */
function test_false_branch_no_narrowing(Animal $obj): void {
    if (get_parent_class($obj) !== 'Animal') {
        /** @mir-check $obj is Animal */
        $_ = $obj;
    }
}

class HasAnimalProp {
    /** @var Animal|Dog */
    public mixed $pet;

    public function testPropStringLiteral(): void {
        if (get_parent_class($this->pet) === 'Animal') {
            /** @mir-check $this->pet is Dog */
            $_ = $this->pet;
        }
    }
}
===expect===
MissingConstructor@30:0-30:21: Class HasAnimalProp has uninitialized properties but no constructor
