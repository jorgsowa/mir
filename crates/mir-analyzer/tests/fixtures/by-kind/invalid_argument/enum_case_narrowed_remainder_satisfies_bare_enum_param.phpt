===description===
FALSE POSITIVE reproducer. Narrowing a variable away from one enum case
via `===` leaves the remaining case-literal union, which must still
satisfy a parameter typed as the bare enum — for both pure and
backed enums.
Expected: no issue.
===config===
php_version=8.1
suppress=UnusedParam
===file===
<?php
enum RoundingMode {
    case Unnecessary;
    case Up;
    case Down;
}

function needsMode(RoundingMode $mode): void {}

function narrowsPureEnum(RoundingMode $mode): void {
    if ($mode === RoundingMode::Unnecessary) {
        return;
    }
    needsMode($mode);
}

enum Suit: string {
    case Hearts = 'H';
    case Spades = 'S';
    case Clubs = 'C';
}

function needsSuit(Suit $suit): void {}

function narrowsBackedEnum(Suit $suit): void {
    if ($suit === Suit::Hearts) {
        return;
    }
    needsSuit($suit);
}
===expect===
