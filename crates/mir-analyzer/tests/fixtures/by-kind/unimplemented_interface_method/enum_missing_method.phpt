===description===
P6(c): Unit enum that implements an interface but is missing the required method emits UnimplementedInterfaceMethod.
===file===
<?php

interface Colorful {
    public function getColor(): string;
}

enum Suit implements Colorful {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Suit must implement Colorful::getColor() from interface
    case Hearts;
    case Diamonds;
}
===expect===
