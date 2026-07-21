===description===
array_key_exists('Ancestor', class_parents($x)) proves $x a STRICT subclass
of Ancestor (same as is_subclass_of($x, Ancestor)) — class_parents()
excludes the receiver's own exact class from its result, unlike
class_implements()/instanceof. Narrowing previously reused plain
instanceof-style narrowing for both, wrongly keeping the exact-class atom
for class_parents(). Covers var/prop/static-prop receivers; false branch
never narrows (mirrors is_subclass_of()'s own convention).
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor,MissingPropertyType
===file===
<?php
class Animal {
    public function onlyOnAnimal(): void {}
}
class Duck extends Animal {
    public function onlyOnDuck(): void {}
}

/** @param Duck|Animal $obj */
function narrowsVarToStrictSubclass(mixed $obj): void {
    if (array_key_exists('Animal', class_parents($obj))) {
        $obj->onlyOnDuck();
    }
}

class Holder {
    /** @var Duck|Animal */
    public mixed $pet;

    public function narrowsPropToStrictSubclass(): void {
        if (array_key_exists('Animal', class_parents($this->pet))) {
            $this->pet->onlyOnDuck();
        }
    }
}

class StaticHolder {
    /** @var Duck|Animal */
    public static mixed $pet;

    public static function narrowsStaticPropToStrictSubclass(): void {
        if (array_key_exists('Animal', class_parents(self::$pet))) {
            self::$pet->onlyOnDuck();
        }
    }
}

/** @param Duck|Animal $obj */
function falseBranchDoesNotNarrow(mixed $obj): void {
    if (!array_key_exists('Animal', class_parents($obj))) {
        // array_key_exists was false — class_parents() narrowing never
        // narrows on the false branch, so $obj stays Duck|Animal here;
        // Animal has no onlyOnDuck(), so this must still be flagged.
        $obj->onlyOnDuck();
    }
}
===expect===
PossiblyInvalidArgument@11:35-11:54: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
PossiblyInvalidArgument@21:39-21:64: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
PossiblyInvalidArgument@32:39-32:64: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
PossiblyInvalidArgument@40:36-40:55: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
UndefinedMethod@44:8-44:26: Method Animal::onlyOnDuck() does not exist
