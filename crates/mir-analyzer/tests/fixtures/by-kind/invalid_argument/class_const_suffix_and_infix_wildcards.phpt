===description===
`self::*_TYPE` and `self::A_*_B` match constants by suffix and infix, like `self::PREFIX_*` by prefix
===file===
<?php
final class Token {
    public const EOF_TYPE = 'end';
    public const NAME_TYPE = 'name';
    public const OTHER = 'other';
    public const MODE_FAST_ON = 1;
    public const MODE_ON = 2;
    public const MODE_SLOW_ON = 4;

    /** @param self::*_TYPE $type */
    public function __construct(public string $type) {}

    /**
     * @param self::MODE_*_ON $mode
     * @param int-mask-of<self::*_ON> $mask
     */
    public function set(int $mode, int $mask): int {
        return $mode | $mask;
    }
}

$t = new Token(Token::NAME_TYPE);
new Token(Token::OTHER);
//        ^^^^^^^^^^^^ InvalidArgument: Argument $type of Token::__construct() expects '"end"|"name"', got '"other"'
$t->set(Token::MODE_SLOW_ON, 7);
$t->set(Token::MODE_ON, 8);
//      ^^^^^^^^^^^^^^ InvalidArgument: Argument $mode of set() expects '1|4', got '2'
//                      ^ InvalidArgument: Argument $mask of set() expects '0|1|2|3|4|5|6|7', got '8'
