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
MethodSignatureMismatch@11:4-11:52: Method FooModel::create() signature mismatch: parameter $x type 'object' is narrower than parent type 'array<mixed, mixed>'
