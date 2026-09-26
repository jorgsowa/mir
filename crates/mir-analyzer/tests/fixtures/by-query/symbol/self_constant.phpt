===description===
`self::CONST` inside a class resolves to that class's constant.
===cursor===
symbol
===file===
<?php
final class Limits {
    public const MAX = 10;
    public function max(): int { return self::M<CURSOR>AX; }
}
===expect===
kind: class constant Limits::MAX
type: 10
