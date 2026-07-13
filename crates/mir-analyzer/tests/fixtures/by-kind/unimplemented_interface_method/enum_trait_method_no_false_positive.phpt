===description===
Enum implementing an interface via a used trait's method does NOT emit UnimplementedInterfaceMethod.
===file===
<?php

trait HasLabel {
    public function getColor(): string {
        return 'red';
    }
}

interface Colorful {
    public function getColor(): string;
}

enum Suit implements Colorful {
    use HasLabel;

    case Hearts;
}
===expect===
