===description===
NonStaticSelfCall fires when a non-static method is called via self:: from a static context.
===file===
<?php
class Counter {
    public function count(): int { return 42; }

    public static function getCount(): int {
        return self::count();
//             ^^^^^^^^^^^^^ NonStaticSelfCall: Non-static method Counter::count() cannot be called statically
    }
}
===expect===
