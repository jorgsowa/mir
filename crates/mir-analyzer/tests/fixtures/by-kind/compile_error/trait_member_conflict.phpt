===description===
A used trait's property or constant must match the class's (or an earlier trait's) declaration: visibility, `static`, `readonly`, native type and default. Identical redeclarations and parent members are fine.
===file===
<?php
trait Counter {
    public int $count = 0;
    protected $label = 'x';
//  ^^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Counter::$label has no type annotation
    public static $shared;
//  ^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Counter::$shared has no type annotation
    public readonly int $id;
    public ?int $limit = null;
    public const MAX = 10;
    final public const MIN = 0;
}

class Same {
//<^^^^^^^^^^^^ MissingConstructor: Class Same has uninitialized properties but no constructor
    use Counter;
    public int $count = 0;
    protected $label = 'x';
//  ^^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Same::$label has no type annotation
    public const MAX = 10;
}

class Base {
    protected $label = 1;
//  ^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Base::$label has no type annotation
}

class FromParent extends Base {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class FromParent has uninitialized properties but no constructor
    use Counter;
}

class Differs {
//<^^^^^^^^^^^^^^^ MissingConstructor: Class Differs has uninitialized properties but no constructor
    use Counter;
//      ^^^^^^^^ CompileError: Differs and Counter define the same property ($count) in the composition of Differs. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
//      ^^^^^^^^ CompileError: Differs and Counter define the same property ($label) in the composition of Differs. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
//      ^^^^^^^^ CompileError: Differs and Counter define the same property ($shared) in the composition of Differs. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
//      ^^^^^^^^ CompileError: Differs and Counter define the same property ($id) in the composition of Differs. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
//      ^^^^^^^^ CompileError: Differs and Counter define the same property ($limit) in the composition of Differs. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
//      ^^^^^^^^ CompileError: Differs and Counter define the same constant (MAX) in the composition of Differs. However, the definition differs and is considered incompatible. Class was composed [trait-constant-conflict]
//      ^^^^^^^^ CompileError: Differs and Counter define the same constant (MIN) in the composition of Differs. However, the definition differs and is considered incompatible. Class was composed [trait-constant-conflict]
    public int $count;
    public $label = 'x';
//  ^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Differs::$label has no type annotation
    public $shared;
//  ^^^^^^^^^^^^^^ MissingPropertyType: Property Differs::$shared has no type annotation
    public int $id;
    public int $limit = 1;
    public const MAX = 11;
    public const MIN = 0;
}

trait Other {
    protected $label = 1;
//  ^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Other::$label has no type annotation
}

class TwoTraits {
//<^^^^^^^^^^^^^^^^^ MissingConstructor: Class TwoTraits has uninitialized properties but no constructor
    use Counter, Other;
//               ^^^^^^ CompileError: Counter and Other define the same property ($label) in the composition of TwoTraits. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
}

trait Scrolling {
    public int $scroll;
}

class Promoted {
    use Scrolling;

    public function __construct(public int $scroll = 5) {}
}
