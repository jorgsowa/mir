===description===
Two string cases with the same value are a PHP fatal; values differing only in case are distinct.
===file===
<?php
namespace App;

enum Suit: string {
    case Hearts = 'h';
    case Spades = 'H';
    case Clubs = 'h';
//               ^^^ DuplicateEnumCaseValue: Duplicate value in enum App\Suit for cases Hearts and Clubs
}
