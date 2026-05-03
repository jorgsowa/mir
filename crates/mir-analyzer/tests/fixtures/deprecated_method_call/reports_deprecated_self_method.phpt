===description===
reports deprecated self method
===file===
<?php
class Greeter {
    /** @deprecated use newGreet() instead */
    public static function oldGreet(string $name): void {}

    public static function test(): void {
        self::oldGreet('Alice');
    }
}
===expect===
UnusedParam@4:36: Parameter $name is never used
DeprecatedMethodCall@7:8: Call to deprecated method Greeter::oldGreet: use newGreet() instead
===ignore===
TODO
