===description===
FALSE POSITIVE reproducer (J2, sibling manifestation). An enum-case literal
produced by narrowing via `===` must satisfy an interface its declaring
enum implements (including the implicit `UnitEnum`/`BackedEnum`), and the
bare `object` type — as an argument and as a property assignment. J1's
subtype fix only covered the exact bare-enum fqcn match; interface targets
went through a separate codebase-aware/argument-checking path that had no
`TLiteralEnumCase` arm at all. Expected: no issue.
===config===
php_version=8.1
suppress=UnusedParam
===file===
<?php
interface HasLabel {
    public function label(): string;
}

enum Status implements HasLabel {
    case Active;
    case Inactive;

    public function label(): string {
        return $this->name;
    }
}

function needsLabel(HasLabel $x): void {}
function needsUnitEnum(UnitEnum $u): void {}
function needsObject(object $o): void {}

function passArg(Status $s): void {
    if ($s === Status::Active) {
        needsLabel($s);
        needsUnitEnum($s);
        needsObject($s);
    }
}

class Holder {
    public HasLabel $thing;

    public function __construct(Status $s) {
        if ($s === Status::Active) {
            $this->thing = $s;
        } else {
            $this->thing = $s;
        }
    }
}

enum Suit: string implements HasLabel {
    case Hearts = 'H';
    case Spades = 'S';

    public function label(): string {
        return $this->value;
    }
}

function passBackedArg(Suit $suit): void {
    if ($suit === Suit::Hearts) {
        needsLabel($suit);
    }
}
===expect===
