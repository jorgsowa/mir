===description===
P6(c): Enum that provides all required interface methods is not reported.
===file===
<?php

interface Colorful {
    public function getColor(): string;
}

enum Suit implements Colorful {
    case Hearts;
    case Diamonds;

    public function getColor(): string
    {
        return match ($this) {
            Suit::Hearts => 'red',
            Suit::Diamonds => 'red',
        };
    }
}
===expect===
