===description===
UnhandledMatchCondition fires when a match on a pure enum misses cases.
===file===
<?php
enum Direction { case North; case South; case East; case West; }

function label(Direction $d): string {
    return match($d) {
        Direction::North => "north",
        Direction::South => "south",
    };
}
===expect===
UnhandledMatchCondition@5:11-8:12: Unhandled match condition: Direction::East, Direction::West
