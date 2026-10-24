===description===
Taint passes through a namespaced function called by its fully qualified name
or through a first-class callable of an imported or fully qualified name.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Foo {
    function bar(string $s): string { return $s; }
}

namespace Other\Foo {
    function bar(int $s): int { return $s; }
}

namespace App {
    use function Foo\bar;
    use Other\Foo;

    function direct(): void {
        echo \Foo\bar($_GET['a']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function importedCallable(): void {
        $f = bar(...);
        echo $f($_GET['b']);
//      ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function fullyQualifiedCallable(): void {
        $f = \Foo\bar(...);
        echo $f($_GET['c']);
//      ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
