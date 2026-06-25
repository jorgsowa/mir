===description===
P6(c): Unit enum without explicit cases() does NOT emit UnimplementedInterfaceMethod — cases() is synthesized by the runtime, not checked.
===file===
<?php

enum Direction {
    case North;
    case South;
    case East;
    case West;
}

// cases() is synthesized by the runtime — no UnimplementedInterfaceMethod is emitted
Direction::cases();
===expect===
