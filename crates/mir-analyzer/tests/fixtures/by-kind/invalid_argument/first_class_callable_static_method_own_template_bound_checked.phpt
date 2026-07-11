===description===
A static method's own `@template T of Base` is no longer left as a bare,
unchecked template placeholder inside its first-class-callable closure —
calling the closure with an argument that violates T's bound is now caught,
matching what a direct `Box::make(new NotBase())` call already catches.
===config===
suppress=UnusedParam
===file===
<?php
class Base {}
class NotBase {}
class Box {
    /**
     * @template T of Base
     * @param T $item
     */
    public static function make($item): void {}
}

$fn = Box::make(...);
$fn(new NotBase());
===expect===
InvalidArgument@13:4-13:17: Argument $item of {closure}() expects 'Base', got 'NotBase'
