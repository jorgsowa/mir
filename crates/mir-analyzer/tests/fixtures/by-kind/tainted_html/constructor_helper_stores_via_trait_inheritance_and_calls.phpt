===description===
A constructor's helper call still stores the argument when the helper comes
from a trait or a parent class, takes the argument by name or through a
spread, appends it to an array property, or runs in a method that reads the
property afterwards.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
trait Stores {
    private function init($v): void { $this->n = $v; }
}
final class UsesTrait {
    use Stores;
    public $n;
    public function __construct($n) { $this->init($n); }
}
abstract class P {
    public $n;
    protected function init($v): void { $this->n = $v; }
}
final class Inherits extends P {
    public function __construct($n) { $this->init($n); }
}
final class Named {
    public $n;
    public function __construct($n) { $this->init(v: $n); }
    private function init($v): void { $this->n = $v; }
}
final class Spread {
    public $n;
    public function __construct(...$a) { $this->init(...$a); }
    private function init($v): void { $this->n = $v; }
}
final class Same {
    public $n;
    public function run($x): void {
        $this->store($x);
        echo $this->n;
    }
    private function store($v): void { $this->n = $v; }
}
final class Arr {
    public $n = [];
    public function __construct($n) { $this->add($n); }
    private function add($v): void { $this->n[] = $v; }
}
function t(): void {
    echo (new UsesTrait($_GET['a']))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new Inherits($_GET['a']))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new Named($_GET['a']))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new Spread($_GET['a']))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    (new Same)->run($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new Arr($_GET['a']))->n[0];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
