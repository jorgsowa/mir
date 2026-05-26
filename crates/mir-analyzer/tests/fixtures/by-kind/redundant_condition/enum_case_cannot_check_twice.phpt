===description===
Type narrowing for specific enum case enables more precise type analysis
===file===
<?php
enum Suit {
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}

// Narrowing allows correct type checking when passed to function
function processClubs(Suit $s): void {
    if ($s === Suit::Clubs) {
        useClubsCase($s);
    }
}

/**
 * @param Suit::Clubs $clubs
 */
function useClubsCase($clubs): void {
    if ($clubs === Suit::Clubs) {
        echo "Got clubs";
    }
}

// Narrowing works for hearts case too
function processHearts(Suit $s): void {
    if ($s === Suit::Hearts) {
        useHeartsCase($s);
    }
}

/**
 * @param Suit::Hearts $hearts
 */
function useHeartsCase($hearts): void {
    if ($hearts === Suit::Hearts) {
        echo "Got hearts";
    }
}

// Narrowing narrows to the specific case inside the if block
function narrowToSpades(Suit $s): void {
    if ($s === Suit::Spades) {
        // At this point, $s is Suit::Spades, not just Suit
        checkSpades($s);
    }
}

/**
 * @param Suit::Spades $spades
 */
function checkSpades($spades): void {
    if ($spades === Suit::Spades) {
        echo "Got spades";
    }
}

processClubs(Suit::Clubs);
processHearts(Suit::Hearts);
narrowToSpades(Suit::Spades);
===expect===
