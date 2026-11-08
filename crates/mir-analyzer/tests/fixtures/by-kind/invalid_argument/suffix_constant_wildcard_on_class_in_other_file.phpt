===description===
A `J::*_SUFFIX` docblock pattern on a class declared in another file matches its constants.
===file:J.php===
<?php
namespace App;
final class J { public const LAX_SCHEMA = 1; public const AUTH_SCHEMA = 3; public const OTHER = 4; }
===file:F.php===
<?php
namespace App;
final class F {
    /** @param J::*_SCHEMA $schema */
    private static function v(int $schema): int { return $schema; }
    public static function f(): void {
        self::v(J::AUTH_SCHEMA);
        self::v(J::OTHER);
//              ^^^^^^^^ InvalidArgument: Argument $schema of v() expects '1|3', got '4'
    }
}
