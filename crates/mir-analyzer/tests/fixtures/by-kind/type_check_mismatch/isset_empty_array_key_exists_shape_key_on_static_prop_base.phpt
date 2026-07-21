===description===
isset()/!empty()/array_key_exists() shape-key narrowing on a nested array
access now also works when the array base is a static-property receiver
(`self::$data['key']`), not just a plain variable or instance property.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MixedArgument
===file===
<?php
class Config {
    /** @var array{name?: string} */
    public static array $data = [];

    /** @var array{sub?: array{name?: string}} */
    public static array $nested = [];

    public static function issetNarrowsStaticProp(): string {
        if (isset(self::$data['name'])) {
            /** @mir-check self::$data is array{name: string} */
            $_ = 1;
            return self::$data['name'];
        }
        return 'default';
    }

    public static function notEmptyNarrowsStaticProp(): string {
        if (!empty(self::$data['name'])) {
            /** @mir-check self::$data is array{name: non-empty-string} */
            $_ = 1;
            return self::$data['name'];
        }
        return 'default';
    }

    public static function arrayKeyExistsNarrowsStaticProp(): string {
        if (array_key_exists('name', self::$data)) {
            /** @mir-check self::$data is array{name: string} */
            $_ = 1;
            return self::$data['name'];
        }
        return 'default';
    }

    public static function arrayKeyExistsNarrowsNestedStaticPropBase(): string {
        if (array_key_exists('name', self::$nested['sub'])) {
            /** @mir-check self::$nested is array{sub: array{name: string}} */
            $_ = 1;
            return self::$nested['sub']['name'];
        }
        return 'default';
    }
}
===expect===
PossiblyNullArgument@37:37-37:57: Argument $array of array_key_exists() might be null
