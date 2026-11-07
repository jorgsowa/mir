===description===
A guard on `$obj->prop` clears it until a call may change the object again.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public $x;
    public function __construct() { $this->x = $_GET['x']; }
    public function reset(): void { $this->x = $_GET['y']; }

    public function guarded(A $o): void {
        if (ctype_digit($this->x)) { echo $this->x; }
        if (ctype_digit($o->x)) { echo $o->x; }
    }

    public function afterMerge(): void {
        if (ctype_digit($this->x)) { echo 'a'; }
        echo $this->x;
//      ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    public function afterCall(): void {
        if (!ctype_digit($this->x)) { return; }
        $this->reset();
        echo $this->x;
//      ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
