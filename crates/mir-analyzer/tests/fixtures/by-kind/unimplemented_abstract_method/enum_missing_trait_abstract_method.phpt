===description===
An enum must implement the abstract methods of the traits it uses, as a class must.
===file===
<?php
trait HasLabel {
    abstract public function label(): string;
}

enum Suit {
//<^^^^^^^^^^^ UnimplementedAbstractMethod: Class Suit must implement abstract method label()
    use HasLabel;
    case Hearts;
}
