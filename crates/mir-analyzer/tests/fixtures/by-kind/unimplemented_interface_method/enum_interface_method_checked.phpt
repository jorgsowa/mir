===description===
P6(c): Enums that fully implement required interface methods are not reported. Backed enum satisfies both the custom interface (with concrete own_methods) and the synthesized BackedEnum methods (cases/from/tryFrom) without any false positives.
===file===
<?php

interface Colorable {
    public function color(): string;
}

enum Suit: string implements Colorable {
    case Hearts = 'H';
    case Spades = 'S';

    public function color(): string
    {
        return match ($this) {
            Suit::Hearts => 'red',
            Suit::Spades => 'black',
        };
    }
}
===expect===
