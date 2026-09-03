===description===
Negative control for the enum-case/bare-enum subtype fix: an enum-case
literal from one enum must still be rejected when passed to a parameter
typed as a completely unrelated enum.
===config===
php_version=8.1
suppress=UnusedParam
===file===
<?php
enum RoundingMode {
    case Unnecessary;
    case Up;
}

enum Suit {
    case Hearts;
    case Spades;
}

function needsSuit(Suit $suit): void {}

function test(): void {
    needsSuit(RoundingMode::Unnecessary);
}
===expect===
InvalidArgument@15:14-15:39: Argument $suit of needsSuit() expects 'Suit', got 'RoundingMode'
