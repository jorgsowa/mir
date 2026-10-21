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
    echo namespace\Box::wrap($_GET['b']);
    echo (new \App\Box($_GET['c']))->get();
    echo (new namespace\Box($_GET['d']))->get();
}
===expect===
TaintedHtml@11:4-11:36: Tainted HTML output — possible XSS
TaintedHtml@12:4-12:41: Tainted HTML output — possible XSS
TaintedHtml@13:4-13:43: Tainted HTML output — possible XSS
TaintedHtml@14:4-14:48: Tainted HTML output — possible XSS
