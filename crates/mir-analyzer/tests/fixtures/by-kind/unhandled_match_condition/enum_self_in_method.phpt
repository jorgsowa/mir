===description===
UnhandledMatchCondition fires when an enum method uses self::Case arms but misses cases.
===file===
<?php
enum Direction {
    case North;
    case South;
    case East;
    case West;

    public function label(): string {
        return match($this) {
            self::North => "north",
            self::South => "south",
        };
    }
}
===expect===
UnhandledMatchCondition@9:15-12:9: Unhandled match condition: Direction::East, Direction::West
