===description===
Go-to-definition on `self::CONST` lands on the constant, not on a method whose name differs only in case.
===ignore===
===cursor===
definition
===file===
<?php
final class Limits {
    public const MAX = 10;
    public function max(): int { return self::M<CURSOR>AX; }
}
===expect===
test.php@3:4-3:26
