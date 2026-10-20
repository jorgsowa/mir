===description===
A property read straight off an inline `new` (parenthesized or not) carries
what the constructor stored there, and a call on it reaches the callee's
property sinks. A constructor that sanitizes, a clean argument and an
untouched property stay clean.
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
final class W {
    public $n;
    public $other = 'x';

    public function __construct($n) {
        $this->n = $n;
    }

    public function render(): void {
        echo $this->n;
    }
}

final class Promoted {
    public function __construct(public string $n) {}
}

final class Escaping {
    public $n;
    public function show(): void { echo $this->n; }

    public function __construct($n) {
        $this->n = htmlspecialchars($n);
    }
}

final class Second {
    public function showA(): void { echo $this->a; }
    public function showB(): void { echo $this->b; }

    public $a;
    public $b;

    public function __construct($a, $b) {
        $this->a = $a;
        $this->b = $b;
    }
}

function reads(): void {
    echo (new W($_GET['a']))->n;

    echo new W($_GET['a'])->n;
    echo (new Promoted($_GET['a']))->n;
    echo (new Escaping($_GET['a']))->n;
    echo (new W('clean'))->n;
    echo (new W($_GET['a']))->other;
    echo (new Second($_GET['a'], 'ok'))->b;
    echo (new Second($_GET['a'], 'ok'))->a;
    echo (new Second('ok', $_GET['a']))->a;
}

function calls(): void {
    (new W($_GET['a']))->render();
    (new W('clean'))->render();
    (new Escaping($_GET['a']))->show();
    (new Second($_GET['a'], 'ok'))->showB();
    (new Second($_GET['a'], 'ok'))->showA();
}
===expect===
TaintedHtml@42:4-42:32: Tainted HTML output — possible XSS
TaintedHtml@44:4-44:30: Tainted HTML output — possible XSS
TaintedHtml@45:4-45:39: Tainted HTML output — possible XSS
TaintedHtml@50:4-50:43: Tainted HTML output — possible XSS
TaintedHtml@55:4-55:33: Tainted HTML output — possible XSS
TaintedHtml@59:4-59:43: Tainted HTML output — possible XSS
