===description===
`array_key_exists('k', self::$prop)` (and `static::$prop`) must narrow a
shape-typed static property's optional key, and the false branch must
exclude a sealed-shape alternative that guarantees the key's presence —
the already-correct instance-property and variable behavior, never wired
for a static-property array_key_exists() dispatch.
===config===
suppress=MissingConstructor,MixedAssignment
===file===
<?php
class Bag {
    /** @var array{name?: string} */
    protected static array $data = [];

    public static function greet(): string {
        if (array_key_exists('name', self::$data)) {
            return self::$data['name'];
        }
        return "unknown";
    }
}

class ChildBag extends Bag {
    public static function greetViaStatic(): string {
        if (array_key_exists('name', static::$data)) {
            return static::$data['name'];
        }
        return "unknown";
    }
}
===expect===
