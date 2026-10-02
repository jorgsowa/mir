===description===
A property default that the `@var` int range excludes is invalid; in-range,
widened and non-literal defaults are fine.
===file===
<?php
final class Color {
    /** @var int<1,255> */
    private int $outOfRange = 0;
//                            ^ InvalidPropertyAssignment: Property $outOfRange expects 'int<1, 255>', cannot assign '0'

    /** @var int<1,255> */
    private static int $staticOutOfRange = 256;
//                                         ^^^ InvalidPropertyAssignment: Property $staticOutOfRange expects 'int<1, 255>', cannot assign '256'

    /** @var int<0,255> */
    private int $widened = 0;

    /** @var int<1,255> */
    private int $inRange = 255;

    /** @var positive-int */
    private int $positive = -1;
//                          ^^ InvalidPropertyAssignment: Property $positive expects 'positive-int', cannot assign '-1'

    /** @var int<1,255>|null */
    private ?int $nullable = null;

    private int $unannotated = 0;

    /** @var int<1,255> */
    private int $fromConstant = self::ZERO;

    private const ZERO = 0;

    public function read(): int {
        /** @mir-check $this->widened is int<0, 255> */
        return $this->widened + $this->inRange + $this->unannotated + $this->nullable;
    }
}
===expect===
