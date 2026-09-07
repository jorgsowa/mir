===description===
`is_subclass_of(self::$prop, X::class)` (and `static::$prop`) must narrow
the static-property receiver like the already-correct instance-property
and variable cases — extract_static_prop_access was never checked at this
dispatch site.
===config===
suppress=MissingConstructor,MissingPropertyType,PossiblyNullArgument
===file===
<?php
class Animal {}
class Dog extends Animal {
    public function bark(): void {}
}

class Container {
    /** @var Animal|Dog|null */
    protected static $pet;

    public static function useIt(): void {
        if (is_subclass_of(self::$pet, 'Animal')) {
            self::$pet->bark();
        }
    }
}

class ChildContainer extends Container {
    public static function useItViaStatic(): void {
        if (is_subclass_of(static::$pet, 'Animal')) {
            static::$pet->bark();
        }
    }
}
===expect===
