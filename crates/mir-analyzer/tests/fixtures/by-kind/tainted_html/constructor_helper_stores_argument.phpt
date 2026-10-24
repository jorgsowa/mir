===description===
A constructor that hands its argument to one of the class's own methods
(`$this->init()`, `self::`, `static::`, `parent::__construct()`, through any
number of hops) leaves the property tainted for the object `new` creates.
Helpers that sanitize, a sanitized or clean argument, a later sanitizing
overwrite and a helper that writes some other property stay clean.
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
final class ViaThis {
    public $n;
    public function __construct($n) { $this->init($n); }
    private function init($v): void { $this->n = $v; }
    public function render(): void { echo $this->n; }
}

final class ViaSelf {
    public $n;
    public function __construct($n) { self::boot($n); }
    private function boot($v): void { $this->n = $v; }
}

final class ViaStatic {
    public $n;
    public function __construct($n) { static::boot($n); }
    private function boot($v): void { $this->n = $v; }
}

final class TwoHops {
    public $n;
    public function __construct($n) { $this->first($n); }
    private function first($a): void { $this->second($a); }
    private function second($b): void { $this->n = $b; }
}

class Base {
    public $n;
    public function __construct($n) { $this->n = $n; }
}
final class Child extends Base {
    public function __construct($n) { parent::__construct($n); }
}

final class SanitizesInHelper {
    public $n;
    public function __construct($n) { $this->init($n); }
    private function init($v): void { $this->n = htmlspecialchars($v); }
}

final class SanitizesBeforeHelper {
    public $n;
    public function __construct($n) { $this->init(htmlspecialchars($n)); }
    private function init($v): void { $this->n = $v; }
}

final class OverwrittenAfterHelper {
    public $n;
    public function __construct($n) {
        $this->init($n);
        $this->n = htmlspecialchars($n);
    }
    private function init($v): void { $this->n = $v; }
}

final class OtherProperty {
    public $n;
    public $other;
    public function __construct($n) { $this->init($n); }
    private function init($v): void { $this->other = $v; }
}

final class SecondParam {
    public $a;
    public $b;
    public function __construct($a, $b) { $this->init($a, $b); }
    private function init($x, $y): void { $this->a = $x; $this->b = $y; }
}

final class ConditionalHelper {
    public $n;
    public function __construct($n, bool $store) { $this->init($n, $store); }
    private function init($v, bool $store): void {
        if ($store) {
            $this->n = $v;
        }
    }
}

final class Recursive {
    public $n;
    public function __construct($n) { $this->loop($n, 3); }
    private function loop($v, int $depth): void {
        if ($depth > 0) {
            $this->loop($v, $depth - 1);
            return;
        }
        $this->n = $v;
    }
}

final class Mutual {
    public $n;
    public function __construct($n) { $this->ping($n); }
    private function ping($v): void { if ($v === '') { return; } $this->pong($v); }
    private function pong($v): void { $this->n = $v; $this->ping(''); }
}

final class FixedDefault {
    public $n;
    public function __construct($n) { $this->init($n); }
    private function init($v, $fallback = 'x'): void { $this->n = $fallback . strlen($v); }
}

function reads(): void {
    $a = new ViaThis($_GET['a']);
    echo $a->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $b = new ViaSelf($_GET['a']);
    echo $b->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $c = new ViaStatic($_GET['a']);
    echo $c->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $d = new TwoHops($_GET['a']);
    echo $d->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $e = new Child($_GET['a']);
    echo $e->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new ViaThis($_GET['a']))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    (new ViaThis($_GET['a']))->render();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function clean(): void {
    echo (new ViaThis('clean'))->n;
    (new ViaThis('clean'))->render();
    echo (new SanitizesInHelper($_GET['a']))->n;
    echo (new SanitizesBeforeHelper($_GET['a']))->n;
    echo (new OverwrittenAfterHelper($_GET['a']))->n;
    echo (new OtherProperty($_GET['a']))->n;
    echo (new Child('clean'))->n;
    echo (new FixedDefault($_GET['a']))->n;
}

function pairs(): void {
    echo (new SecondParam($_GET['a'], 'ok'))->a;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new SecondParam($_GET['a'], 'ok'))->b;
    echo (new SecondParam('ok', $_GET['a']))->a;
    echo (new SecondParam('ok', $_GET['a']))->b;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new OtherProperty($_GET['a']))->other;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function flowy(): void {
    echo (new ConditionalHelper($_GET['a'], true))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new ConditionalHelper('clean', true))->n;
    echo (new Recursive($_GET['a']))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new Mutual($_GET['a']))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
