===description===
SKIPPED-MagicMethodMadeConcreteChecksParams
===file===
<?php
/**
 * @method static void create(array $x)
 */
class Model {
    public static function __callStatic(string $method, array $params) {
    }
}

class FooModel extends Model {
    public static function create(object $x): void {
        $x;
    }
}
===expect===
ImplementedParamTypeMismatch
===ignore===
TODO
