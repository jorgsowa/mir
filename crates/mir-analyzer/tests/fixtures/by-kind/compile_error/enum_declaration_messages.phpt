===description===
Enum declaration errors use PHP's text; an invalid backing type does not also report its case values.
===file===
<?php
namespace App;

enum Pure {
    case A = 1;
//         ^ CompileError: Case A of non-backed enum App\Pure must not have a value [pure-enum-case-value]
    case class;
//       ^^^^^ CompileError: A class constant must not be called 'class'; it is reserved for class name fetching [enum-case-named-class]

    abstract public function label(): string;
//  ^^^^^^^^^^^^^^^ CompileError: Enum method App\Pure::label() must not be abstract [abstract-enum-method]
}

enum Weight: float {
//           ^^^^^ CompileError: Enum backing type must be int or string, float given [invalid-enum-backing-type]
    case Light = 1.5;
}

enum Flag: bool {
//         ^^^^ CompileError: Enum backing type must be int or string, bool given [invalid-enum-backing-type]
    case On = true;
}

enum Code: int {
    case Missing;
//       ^^^^^^^ CompileError: Case Missing of backed enum App\Code must have a value [missing-enum-case-value]
}
