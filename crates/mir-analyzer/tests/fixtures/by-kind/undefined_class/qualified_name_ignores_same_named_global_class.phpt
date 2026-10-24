===description===
A qualified name in code always resolves against the current namespace (or a
`use` import of its first segment), even when a class with the written name
exists globally: `Foo\Bar` in `namespace App` is `App\Foo\Bar`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Foo {
    class Bar {}
    class Err extends \Exception {}
}

namespace App {
    final class C extends Foo\Bar {}

    function f(Foo\Bar $b, mixed $m): void {
        $x = new Foo\Bar();
        if ($m instanceof Foo\Bar) {}
        try {} catch (Foo\Err $e) {}
    }
}

namespace Other {
    use Foo;

    final class D extends Foo\Bar {}

    function g(Foo\Bar $b): void {
        $x = new Foo\Bar();
        try {} catch (Foo\Err $e) {}
    }
}
===expect===
UndefinedClass@8:26-8:33: Class App\Foo\Bar does not exist
UndefinedClass@10:15-10:22: Class App\Foo\Bar does not exist
UndefinedClass@11:17-11:24: Class App\Foo\Bar does not exist
UndefinedClass@12:26-12:33: Class App\Foo\Bar does not exist
UndefinedClass@13:22-13:29: Class App\Foo\Err does not exist
