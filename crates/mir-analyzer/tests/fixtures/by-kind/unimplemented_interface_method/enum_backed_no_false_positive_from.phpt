===description===
P6(c): Backed enum without explicit from()/tryFrom() does NOT emit UnimplementedInterfaceMethod — these are synthesized by the runtime.
===file===
<?php

enum Priority: int {
    case Low = 1;
    case Medium = 2;
    case High = 3;
}

// from()/tryFrom() are synthesized by the runtime — no UnimplementedInterfaceMethod is emitted
Priority::from(2);
Priority::tryFrom(99);
===expect===
