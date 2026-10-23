===description===
A fully qualified source call (`\getallheaders()`, `\filter_input()`) taints a
function's return value and a stored property, like the unqualified form.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <PossiblyInvalidArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

function headers() { return \getallheaders(); }
function input() { return \filter_input(INPUT_GET, 'x'); }

class Box {
    public $v;
    public function fill(): void { $this->v = \getallheaders()['x']; }
    public function get() { return $this->v; }
}

function echo_headers(): void {
    echo headers()['x'];
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function echo_input(): void {
    echo input();
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function echo_property(Box $b): void {
    echo $b->get();
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
