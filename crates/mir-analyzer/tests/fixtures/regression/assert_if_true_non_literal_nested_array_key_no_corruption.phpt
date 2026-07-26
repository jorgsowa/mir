===description===
`@psalm-assert-if-true string $config['a'][self::KEY]` (a nested path whose
INNER key is non-literal) must leave the whole assertion a no-op, not
partially split the path and corrupt the whole `$config` parameter's type
— same no-corruption guarantee the single-key case already has, extended
to a multi-segment path.
===config===
suppress=MissingReturnType,MixedArgument,UnusedParam
===file===
<?php
class Config {
    const KEY = 'b';

    /**
     * @param array{a?: array{b?: string}} $config
     * @psalm-assert-if-true string $config['a'][self::KEY]
     */
    public static function hasNestedStatus(array $config): bool {
        return isset($config['a'][self::KEY]);
    }
}

/** @param array{a?: array{b?: string}} $c */
function test(array $c): void {
    if (Config::hasNestedStatus($c)) {
        /** @mir-check $c is array{a?: array{b?: string}} */
        $_ = 1;
    }
}
===expect===
