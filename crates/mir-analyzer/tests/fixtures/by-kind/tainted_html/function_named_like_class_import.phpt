===description===
A function declared under the same name as a class import (`use Lib\Helper;`
then `function helper()`) is keyed by the current namespace, so its return and
parameter taint is not lost.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Lib {
    class Helper {}
    class Source {}
}

namespace App {
    use Lib\Helper;
    use Lib\Source;

    function helper(string $x): string { return $x; }
    function source(): string { return $_GET['a']; }
//                              ^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'

    function f(): void {
        echo helper($_GET['a']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo source();
//      ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
