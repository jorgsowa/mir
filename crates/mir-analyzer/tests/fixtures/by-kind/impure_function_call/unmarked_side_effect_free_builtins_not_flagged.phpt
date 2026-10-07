===description===
Builtins with no side effects beyond writing a local by-ref output are pure
even though their stubs carry no @pure tag.
===file===
<?php
final class Codec {
    /** @pure */
    public static function decode(string $hex): string {
        $bin = hex2bin($hex);
        assert($bin !== false);
        /** @mir-check $bin is string */
        return $bin;
    }

    /** @pure */
    public static function digits(string $value): ?string {
        if (preg_match('/^(\d+)$/', $value, $matches) !== 1) {
            return null;
        }
        /** @mir-check $matches is list<string> */
        return $matches[1];
    }

    /** @pure */
    public static function isInvokable(mixed $value): bool {
        return is_callable($value, false, $name);
    }

    /**
     * @pure
     * @param list<int> $values
     * @return list<int>
     */
    public static function sorted(array $values): array {
        sort($values);
        return $values;
    }

    /** @pure */
    public static function dashes(string $value): string {
        $out = str_replace(' ', '-', $value, $count);
        /** @mir-check $count is int */
        return $out;
    }
}
