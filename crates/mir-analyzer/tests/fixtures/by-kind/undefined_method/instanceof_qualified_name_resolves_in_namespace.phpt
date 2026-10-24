===description===
`$m instanceof Foo\Bar` in `namespace App` narrows to `App\Foo\Bar`, not to a
global `Foo\Bar` that also exists.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Foo {
    class Bar { public function globalOnly(): void {} }
}
namespace App\Foo {
    class Bar { public function appOnly(): void {} }
}
namespace App {
    function f(mixed $m): void {
        if ($m instanceof Foo\Bar) {
            $m->appOnly();
            $m->globalOnly();
        }
    }
}
===expect===
UndefinedMethod@12:12-12:28: Method App\Foo\Bar::globalOnly() does not exist
