===description===
Class constants whose initializer references another already-collected
same-file class-like's enum case or plain constant infer that member's real
type instead of `mixed`, so a compatible downstream use is not flagged.
===config===
suppress=UnusedParam
===file===
<?php
enum Suit {
    case Hearts;
    case Spades;
}
class Card {
    const DEFAULT = Suit::Hearts;
}
function takesSuit(Suit $s): void {}
function useCard(): void {
    takesSuit(Card::DEFAULT);
}

class Base {
    const BAR = 5;
}
class Derived {
    const COPY = Base::BAR;
}
function takesInt(int $i): void {}
function useDerived(): void {
    takesInt(Derived::COPY);
}
===expect===
