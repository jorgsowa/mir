===description===
reports deprecated self method
===file===
<?php
class Greeter {
    /** @deprecated use newGreet() instead */
    public static function oldGreet(string $name): void {}
//                                  ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used

    public static function test(): void {
        self::oldGreet('Alice');
//      ^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedMethodCall: Call to deprecated method Greeter::oldGreet: use newGreet() instead
    }
}
