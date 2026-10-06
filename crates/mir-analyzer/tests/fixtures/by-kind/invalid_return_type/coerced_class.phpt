===description===
Coerced class
===file===
<?php
class NullableClass {
}

class NullableBug {
    /**
     * @param class-string|null $className
     * @return object|null
     */
    public static function mock($className) {
        if (!$className) { return null; }
        return new $className();
    }

    /**
     * @return ?NullableClass
     */
    public function returns_nullable_class() {
        /** @suppress ArgumentTypeCoercion */
//                    ^^^^^^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'ArgumentTypeCoercion' is never used
        return self::mock("NullableClass");
    }
}
===expect===
