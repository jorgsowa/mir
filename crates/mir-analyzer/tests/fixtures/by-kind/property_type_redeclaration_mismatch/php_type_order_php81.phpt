===description===
Below 8.2 a native `iterable` keeps its name in the expected type.
===config===
<mir>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
interface I {}
interface J {}
class A {}
class P {
//<^^^^^^^^^ MissingConstructor: Class P has uninitialized properties but no constructor
    public int|string|null $a;
    public array|bool|float $b;
    public null|self|A $c;
    public iterable|string|A $d;
    public ?iterable $h;
}
class Q extends P {
//<^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class Q has uninitialized properties but no constructor
    public $a;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$a has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$a must be string|int|null (as in class P)
    public $b;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$b has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$b must be array|float|bool (as in class P)
    public $c;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$c has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$c must be P|A|null (as in class P)
    public $d;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$d has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$d must be A|iterable|string (as in class P)
    public $h;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$h has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$h must be ?iterable (as in class P)
}
