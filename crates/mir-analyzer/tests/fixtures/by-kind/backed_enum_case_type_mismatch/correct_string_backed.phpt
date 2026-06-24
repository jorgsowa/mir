===description===
P6(a): A string-backed enum where all cases have string values must produce no errors.
===file===
<?php
enum Direction: string {
    case North = 'north';
    case South = 'south';
    case East = 'east';
    case West = 'west';
}
===expect===
