===description===
Declaration errors the parser reports use PHP's text, with the class and member PHP names.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
namespace App;

abstract class Shape {
    final abstract public function area(): float;
//  ^^^^^^^^^^^^^^^^^^^^^ ParseError: Cannot use the final modifier on an abstract method [abstract-final-conflict]

    abstract public $name;
//  ^^^^^^^^^^^^^^^^^^^^^ CompileError: Only hooked properties may be declared abstract [abstract-property-without-abstract-hook]
//  ^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property App\Shape::$name has no type annotation

    static const SIDES = 0;
//         ^^^^^ ParseError: Cannot use the static modifier on a class constant [invalid-constant-modifier]

    final private const SECRET = 1;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Private constant App\Shape::SECRET cannot be final as it is not visible to other classes [private-final-constant]
}

final class Sized {
    private public(set) int $size = 0;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Visibility of property App\Sized::$size must not be weaker than set visibility [set-visibility-wider-than-visibility]
}

final class Box {
    public int $width {
        get() => 1;
//         ^ CompileError: get hook of property App\Box::$width must not have a parameter list [get-hook-parameters]
    }

    public int $height {
        set(int $a, int $b) {}
//         ^ CompileError: set hook of property App\Box::$height must accept exactly one parameters [set-hook-parameter-count]
    }
}
