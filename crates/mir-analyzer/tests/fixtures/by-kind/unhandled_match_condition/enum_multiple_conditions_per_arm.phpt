===description===
Multiple conditions in one match arm all count as covered cases.
===file===
<?php
enum Direction { case North; case South; case East; case West; }

function label(Direction $d): string {
    return match($d) {
        Direction::North, Direction::South => "vertical",
        Direction::East, Direction::West   => "horizontal",
    };
}
===expect===
