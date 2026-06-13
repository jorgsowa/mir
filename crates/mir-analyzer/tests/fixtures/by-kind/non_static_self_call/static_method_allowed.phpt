===description===
NonStaticSelfCall does NOT fire when calling a static method via self::.
===file===
<?php
class Counter {
    public static function increment(int $n): int { return $n + 1; }

    public static function run(): int {
        return self::increment(0);
    }
}
===expect===
