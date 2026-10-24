===description===
Reading an undeclared static property is reported; isset, traits, static::, class_exists-guarded and unknown-ancestor classes are not
===file===
<?php
class Base { public static int $inherited = 0; }
class A extends Base {
    public static int $declared = 0;

    public function read(): int {
        echo self::$nope;
//                 ^^^^^ UndefinedProperty: Property A::$nope does not exist
        echo parent::$nopeParent;
//                   ^^^^^^^^^^^ UndefinedProperty: Property Base::$nopeParent does not exist
        echo static::$lateBound;
        echo isset(self::$maybe) ? 1 : 0;
        return self::$declared + self::$inherited + A::$inherited;
    }
}
trait T {
    public function read(): mixed { return self::$fromUser; }
}
class Unknown extends NotLoaded {}
//                    ^^^^^^^^^ UndefinedClass: Class NotLoaded does not exist
function f(): void {
    echo A::$missing;
//          ^^^^^^^^ UndefinedProperty: Property A::$missing does not exist
    echo Unknown::$whatever;
    if (class_exists(Optional::class, false) && Optional::$x) {}
}
