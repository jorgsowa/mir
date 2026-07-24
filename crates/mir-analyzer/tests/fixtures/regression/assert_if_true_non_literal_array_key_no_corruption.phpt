===description===
`@psalm-assert-if-true string $config[self::KEY]` (a non-literal, class-
constant array key) used to silently corrupt the WHOLE `$config`/`$c`
parameter's type instead of no-op'ing: `split_array_key_suffix` stripped
the bracket suffix off the name regardless of whether the key inside
parsed as a literal, so a non-literal key still produced the bare name
`"config"` (a real match) paired with a `None` key, which
`apply_assertions` reads as "target the whole variable" — turning a
harmless unsupported case into an active false-positive generator.
===config===
suppress=MissingReturnType,MixedArgument,UnusedParam
===file===
<?php
class Config {
    const KEY = 'status';

    /**
     * @param array{status?: string} $config
     * @psalm-assert-if-true string $config[self::KEY]
     */
    public static function hasStatus(array $config): bool {
        return isset($config[self::KEY]);
    }
}

/** @param array{status?: string} $c */
function test(array $c): void {
    if (Config::hasStatus($c)) {
        /** @mir-check $c is array{status?: string} */
        $_ = 1;
    }
}
===expect===
