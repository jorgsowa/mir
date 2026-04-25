===file===
<?php
class Math {
    public static function sq(int $n): int { return $n * $n; }
}
function test(): void {
    Math :: sq(3);
    Math  ::  sq(3);
}
===expect===
