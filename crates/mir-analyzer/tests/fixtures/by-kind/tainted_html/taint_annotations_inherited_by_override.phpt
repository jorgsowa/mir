===description===
An overriding method without taint annotations takes those of the method it overrides.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface I {
    /** @psalm-taint-escape html */
    public function esc(string $v): string;
    /** @psalm-taint-source input */
    public function src(): string;
}
abstract class P implements I {}
class E extends P {
    public function esc(string $v): string { return $v; }
    public function src(): string { return 'x'; }
}
function g(E $e): void {
    echo $e->esc($_GET['x']);
    echo $e->src();
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
