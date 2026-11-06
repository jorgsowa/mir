===description===
`self::`/`static::` in an anonymous class reach its own members and those it inherits.
===file===
<?php
interface I { const C = 1; }
class P implements I { public static function s(): int { return 1; } }
class Q extends P {}
function f(): void {
    new class extends Q {
        const E = 2;
        public static function own(): int { return 3; }
        public function f(): int {
            return self::C + static::C + self::E + self::s() + static::s() + self::own();
        }
    };
    new class implements I {
        public function f(): int {
            return self::C;
        }
    };
}
