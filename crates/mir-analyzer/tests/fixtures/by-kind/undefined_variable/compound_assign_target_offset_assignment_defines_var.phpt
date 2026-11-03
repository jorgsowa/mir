===description===
A compound assignment evaluates its target's offsets before the value, so a variable assigned in the offset is defined on the right-hand side.
===file===
<?php
final class Registry {
    /** @var array<string, string> */
    public static array $names = [];

    public static function get(string $class): string {
        return self::$names[$name = strtolower($class)] ??= $name;
    }
}

/** @param array<string, int> $counts */
function bump(array $counts, string $key): array {
    $counts[$k = trim($key)] ??= strlen($k);
    return $counts;
}
