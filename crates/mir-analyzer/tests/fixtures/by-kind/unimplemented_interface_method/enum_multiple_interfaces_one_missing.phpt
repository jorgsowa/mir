===description===
P6(c): Enum implementing two interfaces but missing a method from one of them.
===file===
<?php

interface Colorful {
    public function getColor(): string;
}

interface Labeled {
    public function label(): string;
}

enum Suit implements Colorful, Labeled {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Suit must implement Labeled::label() from interface
    case Hearts;
    case Diamonds;

    public function getColor(): string
    {
        return 'red';
    }
    // Missing label()
}
