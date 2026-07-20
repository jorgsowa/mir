===description===
array_key_exists('Iface', class_implements(self::$prop)) / class_parents()
narrow a static-property receiver, the static-property counterpart of the
already-existing plain-variable/instance-property narrowing.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
interface Quacks {}
class Animal {}
class Duck extends Animal implements Quacks {}
class Cat extends Animal {}
class Rock {}

class Holder {
    /** @var Animal|Duck */
    public static mixed $pet;

    public static function testStaticPropClassImplementsTrue(): void {
        if (array_key_exists('Quacks', class_implements(self::$pet))) {
            /** @mir-check self::$pet is Animal&Quacks|Duck */
            $_ = self::$pet;
        }
    }

    /** @var Duck|Cat */
    public static mixed $pet2;

    public static function testStaticPropClassImplementsFalse(): void {
        if (!array_key_exists('Quacks', class_implements(self::$pet2))) {
            /** @mir-check self::$pet2 is Cat */
            $_ = self::$pet2;
        }
    }

    /** @var Duck|Rock */
    public static mixed $pet3;

    public static function testStaticPropClassParentsTrue(): void {
        if (array_key_exists('Animal', class_parents(self::$pet3))) {
            /** @mir-check self::$pet3 is Duck */
            $_ = self::$pet3;
        }
    }
}
===expect===
MissingConstructor@8:0-8:14: Class Holder has uninitialized properties but no constructor
PossiblyInvalidArgument@13:39-13:67: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
PossiblyInvalidArgument@23:40-23:69: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
PossiblyInvalidArgument@33:39-33:65: Argument $array of array_key_exists() expects 'array', possibly different type 'array<int, string>|false' provided
