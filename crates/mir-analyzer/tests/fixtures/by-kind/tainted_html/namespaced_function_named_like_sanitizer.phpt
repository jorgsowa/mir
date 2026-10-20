===description===
A namespaced user function named like a builtin sanitizer (or a builtin that
produces safe output) is not that builtin: its result keeps the argument's
taint. The global builtin still sanitizes when called with a leading `\`, from
the global namespace, or from a namespace without its own function.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App {
    function htmlspecialchars(string $s): string { return $s; }
    function base64_encode(string $s): string { return $s; }

    function shadowed(): void {
        echo htmlspecialchars($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo base64_encode($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function explicit_global(): void {
        echo \htmlspecialchars($_GET['q']);
        echo \base64_encode($_GET['q']);
    }
}

namespace Other {
    function untouched(): void {
        echo htmlspecialchars($_GET['q']);
        echo base64_encode($_GET['q']);
    }
}

namespace Imported {
    use function App\htmlspecialchars;

    function through_import(): void {
        echo htmlspecialchars($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo \htmlspecialchars($_GET['q']);
    }
}

namespace {
    function global_scope(): void {
        echo htmlspecialchars($_GET['q']);
    }
}
