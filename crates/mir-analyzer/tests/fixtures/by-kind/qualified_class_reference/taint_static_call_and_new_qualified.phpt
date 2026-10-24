===description===
Taint flows through static calls and `new` written with fully qualified and relative names.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

class Box {
    public function __construct(public string $v) {}
    public static function wrap(string $s): string { return $s; }
    public function get(): string { return $this->v; }
}

function f(): void {
    echo \App\Box::wrap($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo namespace\Box::wrap($_GET['b']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new \App\Box($_GET['c']))->get();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new namespace\Box($_GET['d']))->get();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
