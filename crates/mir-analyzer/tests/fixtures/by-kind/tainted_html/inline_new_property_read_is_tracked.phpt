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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    echo new W($_GET['a'])->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new Promoted($_GET['a']))->n;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new Escaping($_GET['a']))->n;
    echo (new W('clean'))->n;
    echo (new W($_GET['a']))->other;
    echo (new Second($_GET['a'], 'ok'))->b;
    echo (new Second($_GET['a'], 'ok'))->a;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (new Second('ok', $_GET['a']))->a;
}

function calls(): void {
    (new W($_GET['a']))->render();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    (new W('clean'))->render();
    (new Escaping($_GET['a']))->show();
    (new Second($_GET['a'], 'ok'))->showB();
    (new Second($_GET['a'], 'ok'))->showA();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
