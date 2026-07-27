===description===
A class name embedded inside a `Closure(...)`/`callable(...)` signature
never went through `use`-import/namespace resolution at all (unlike
every other type position) — calling a `Closure(Foo): void`-typed
parameter with a genuinely-correct `Foo` instance false-positived,
because the signature's own `Foo` stayed an unresolved bare name instead
of resolving to `App\Models\Foo` (exactly what `new Foo()` resolves to
via the same `use` import).
===file===
<?php
namespace App;

use App\Models\Foo;

class Registry {
    /**
     * @param Closure(Foo): void $f
     */
    public function run(\Closure $f): void {
        $f(new Foo());
    }
}

namespace App\Models;

class Foo {}
===expect===
