===description===
When @param int conflicts with bool PHP hint, the PHP hint wins. Passing a bool is OK;
passing an int fires InvalidArgument (not ArgumentTypeCoercion) because the param is bool.
===config===
suppress=UnusedParam,MismatchingDocblockParamType
===file===
<?php
class Foo {
    /**
     * @param int $x   <-- docblock says int, but PHP hint is bool
     */
    public static function convert(bool $x): void {
        echo $x;
    }
}
Foo::convert(true);  // bool → bool hint: OK
Foo::convert(1);     // int → bool hint: InvalidArgument
===expect===
InvalidArgument@11:13-11:14: Argument $x of convert() expects 'bool', got '1'
