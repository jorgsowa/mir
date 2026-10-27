===description===
PHP 7.4 rejects any return type on `__clone()`, `void` included, in classes, traits and anonymous classes; interface methods are not checked.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
namespace App;

class A {
    function __clone(): void {}
//                      ^^^^ CompileError: Clone method App\A::__clone() cannot declare a return type [magic-method-return-type]
}
trait T {
    function __clone(): void {}
//                      ^^^^ CompileError: Clone method App\T::__clone() cannot declare a return type [magic-method-return-type]
}
interface I {
    function __clone(): void;
}
echo get_class(new class {
    function __clone(): void {}
//                      ^^^^ CompileError: Clone method class@anonymous::__clone() cannot declare a return type [magic-method-return-type]
});
