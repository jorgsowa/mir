===description===
A call to a function declared in another `namespace {}` block of the same file
carries that function's return taint.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace {
    function global_src() { return $_GET['x']; }
}

namespace Lib {
    function lib_src() { return $_GET['x']; }
}

namespace App {
    function unqualified_global(): void {
        echo global_src();
//      ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function fully_qualified(): void {
        echo \Lib\lib_src();
//      ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
