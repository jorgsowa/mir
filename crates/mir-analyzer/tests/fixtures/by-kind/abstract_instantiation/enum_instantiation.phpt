===description===
An enum cannot be instantiated with `new`, also through `self` or `static` inside it; PHP throws `Cannot instantiate enum`.
===file===
<?php
namespace App;

enum Suit {
    case Hearts;

    public static function make(): self {
        return new static();
//                 ^^^^^^ EnumInstantiation: Cannot instantiate enum App\Suit
    }
}

var_dump(new Suit());
//           ^^^^ EnumInstantiation: Cannot instantiate enum App\Suit
