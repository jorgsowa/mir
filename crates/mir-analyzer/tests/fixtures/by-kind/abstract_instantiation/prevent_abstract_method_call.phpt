===description===
Prevent abstract method call
===file===
<?php
abstract class Base {
    public static function callAbstract() : void {
        static::bar();
    }

    abstract static function bar() : void;
}

Base::bar();
===expect===
AbstractMethodCall@10:0-10:11: Cannot call abstract method Base::bar()
