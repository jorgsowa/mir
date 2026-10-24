===description===
Writing an undeclared static property is reported; static::, traits and class_exists-guarded classes are not
===file===
<?php
class A {
    public static int $declared = 0;

    public function write(): void {
        self::$nope = 1;
//            ^^^^^ UndefinedProperty: Property A::$nope does not exist
        A::$nope2 = 2;
//         ^^^^^^ UndefinedProperty: Property A::$nope2 does not exist
        self::$nope3 .= 'x';
//            ^^^^^^ UndefinedProperty: Property A::$nope3 does not exist
        static::$lateBound = 3;
        self::$declared = 4;
        if (class_exists(Optional::class, false)) {
            Optional::$x = 5;
        }
    }
}
trait T {
    public function write(): void { self::$fromUser = 1; }
}
