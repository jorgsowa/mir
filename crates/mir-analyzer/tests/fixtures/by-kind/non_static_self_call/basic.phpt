===description===
NonStaticSelfCall fires when a non-static method is called via self:: from a static context.
===file===
<?php
class Counter {
    public function count(): int { return 42; }

    public static function getCount(): int {
        return self::count();
    }
}
===expect===
NonStaticSelfCall@6:15-6:28: Non-static method Counter::count() cannot be called statically
