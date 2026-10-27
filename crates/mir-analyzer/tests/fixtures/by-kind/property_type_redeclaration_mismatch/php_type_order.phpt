===description===
The expected type is printed in PHP's order: intersections, classes, then builtins; `self` resolved, `iterable` as `Traversable|array` from 8.2.
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
    public (I&J)|string|A $e;
    public (I&J)|null $f;
    public false|null $g;
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
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$d must be Traversable|A|array|string (as in class P)
    public $e;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$e has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$e must be (I&J)|A|string (as in class P)
    public $f;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$f has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$f must be (I&J)|null (as in class P)
    public $g;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$g has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$g must be ?false (as in class P)
    public $h;
//  ^^^^^^^^^ MissingPropertyType: Property Q::$h has no type annotation
//  ^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of Q::$h must be Traversable|array|null (as in class P)
}
