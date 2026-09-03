===description===
`class_exists(self::$prop)`/`interface_exists(self::$prop)` narrow a
static-property argument to class-string/interface-string, like their
var/instance-property counterparts already do.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class Foo {}
interface Bar {}

class Container {
    private static string $className = 'Foo';
    private static string $interfaceName = 'Bar';

    public static function testClassExists(): void {
        if (class_exists(self::$className)) {
            /** @mir-check self::$className is class-string */
            $_ = self::$className;
        }
    }

    public static function testInterfaceExists(): void {
        if (interface_exists(self::$interfaceName)) {
            /** @mir-check self::$interfaceName is interface-string */
            $_ = self::$interfaceName;
        }
    }
}
===expect===
