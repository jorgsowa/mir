===description===
Readonly property errors name the property as PHP does, including anonymous classes and traits.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
namespace App;

final class Untyped {
    public readonly $a;
//  ^^^^^^^^^^^^^^^^^^ CompileError: Readonly property App\Untyped::$a must have type [untyped-readonly-property]
}

final class Promoted {
    public function __construct(public readonly $b) {}
//                              ^^^^^^^^^^^^^^^ CompileError: Readonly property App\Promoted::$b must have type [untyped-readonly-property]
}

readonly class Point {
    public static int $origin = 0;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Static property App\Point::$origin cannot be readonly [static-readonly-property]
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReadonlyPropertyDeclaration: Readonly property App\Point::$origin cannot have default value

    public function __construct(public int $x) {}
}

trait HasId {
    public readonly $id;
//  ^^^^^^^^^^^^^^^^^^^ CompileError: Readonly property App\HasId::$id must have type [untyped-readonly-property]
}

function make(): object {
    return new class {
        public readonly $q;
//      ^^^^^^^^^^^^^^^^^^ CompileError: Readonly property class@anonymous::$q must have type [untyped-readonly-property]
    };
}

final class Hooked {
    public readonly int $h { get => 1; }
//                           ^^^^^^^^^ CompileError: Hooked properties cannot be readonly [readonly-property-hooks]
}
