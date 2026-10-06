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
//             ^ +3:9 UnhandledMatchCondition: Unhandled match condition: Direction::East, Direction::West
            self::North => "north",
            self::South => "south",
        };
    }
}
