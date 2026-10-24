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
//                        ^^^^^^^ UndefinedClass: Class App\Foo\Bar does not exist

    function f(Foo\Bar $b, mixed $m): void {
//             ^^^^^^^ UndefinedClass: Class App\Foo\Bar does not exist
        $x = new Foo\Bar();
//               ^^^^^^^ UndefinedClass: Class App\Foo\Bar does not exist
        if ($m instanceof Foo\Bar) {}
//                        ^^^^^^^ UndefinedClass: Class App\Foo\Bar does not exist
        try {} catch (Foo\Err $e) {}
//                    ^^^^^^^ UndefinedClass: Class App\Foo\Err does not exist
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
