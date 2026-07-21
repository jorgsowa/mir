===description===
array_key_exists(self::$key, $arr) resolves the key when it's a static
property already narrowed to a single literal, same as a plain variable or
instance property already does — literal_key resolution had no static-prop arm.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Holder {
    /** @var 'favicon' */
    public static string $key = 'favicon';

    /** @var string */
    public static string $unnarrowedKey = 'x';

    /** @param array{title: string} $meta */
    public function guardedByStaticPropStringKey(array $meta): string {
        return array_key_exists(self::$key, $meta) ? (string) $meta['favicon'] : '';
    }

    /** @param array{title: string} $meta */
    public function notNarrowedWhenKeyIsNotALiteral(array $meta): string {
        return array_key_exists(self::$unnarrowedKey, $meta) ? (string) $meta['favicon'] : '';
    }
}
===expect===
NonExistentArrayOffset@16:78-16:87: Array offset 'favicon' does not exist
