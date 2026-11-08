===description===
An element of a property written with a literal key holds what was written, not its siblings' taint.
===file===
<?php
class A {
    public array $d = [];
    public function f(): void {
        $this->d['p'] = 'c';
        $this->d['q'] = $_GET['x'];
//                      ^^^^^^^^^^ MixedArrayAccess: Array access on mixed type
        echo $this->d['p'];
        echo $this->d['q'];
//      ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
