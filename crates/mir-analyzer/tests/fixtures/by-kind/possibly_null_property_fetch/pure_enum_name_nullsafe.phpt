===description===
pure enum name nullsafe
===file===
<?php
enum Direction {
    case North;
    case South;
}
function test(?Direction $dir): string {
    return $dir?->name ?? 'none';
}
===expect===
