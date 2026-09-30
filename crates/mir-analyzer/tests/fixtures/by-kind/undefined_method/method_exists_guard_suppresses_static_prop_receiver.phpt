===description===
method_exists(self::$prop, 'method') guards an instance-method call on the
same static-property receiver; the guard has no effect once outside the
branch, or for property_exists() (methods/properties are independent
namespaces).
===config===
suppress=MissingReturnType,MissingConstructor
===file===
<?php
class Bar {}

class Registry {
    private static Bar $factory;

    public static function build(): void {
        if (method_exists(self::$factory, 'create')) {
            self::$factory->create();
        }
    }

    public static function buildOutsideGuard(): void {
        self::$factory->create();
//      ^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Bar::create() does not exist
    }

    public static function propertyExistsDoesNotGuardMethod(): void {
        if (property_exists(self::$factory, 'create')) {
            self::$factory->create();
//          ^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Bar::create() does not exist
        }
    }
}
===expect===
