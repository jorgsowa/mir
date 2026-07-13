===description===
No ImplicitToStringCast when the enum implements Stringable.
===config===
suppress=UnusedVariable
===file===
<?php
enum Suit implements Stringable {
    case Hearts;

    public function __toString(): string {
        return 'Hearts';
    }
}
$s = 'Suit: ' . Suit::Hearts;
echo Suit::Hearts;
===expect===
