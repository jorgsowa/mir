===description===
list<self> inside an enum accepts its own cases
===file===
<?php
enum Suit {
    case Hearts;
    case Spades;

    /** @return list<self> */
    public static function all(): array {
        return [self::Hearts, self::Spades];
    }

    /** @return list<static> */
    public static function allStatic(): array {
        return [self::Hearts, self::Spades];
    }
}

/** @mir-check $all is list<Suit> */
$all = Suit::all();
//<^^^^^^^^^^^^^^^^^^^ TypeCheckMismatch: Type of $all is expected to be list<Suit>, got mixed
echo count($all);
