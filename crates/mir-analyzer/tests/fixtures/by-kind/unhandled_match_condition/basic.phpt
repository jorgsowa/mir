===description===
UnhandledMatchCondition fires when a match on a pure enum misses cases.
===file===
<?php
enum Direction { case North; case South; case East; case West; }

function label(Direction $d): string {
    return match($d) {
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: Direction::East, Direction::West
        Direction::North => "north",
        Direction::South => "south",
    };
}
===expect===
