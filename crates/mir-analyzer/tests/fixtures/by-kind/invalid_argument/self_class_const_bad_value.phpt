===description===
Self class const bad value
===ignore===
TODO
===file===
<?php
class A {
    const FOO = "foo";
    const BAR = "bar";

    /**
     * @param (self::FOO | self::BAR) $s
     */
    public static function foo(string $s) : void {}
}

A::foo("for");
===expect===
