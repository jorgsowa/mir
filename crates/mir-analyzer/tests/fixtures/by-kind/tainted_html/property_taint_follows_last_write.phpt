===description===
A property a constructor overwrites holds the taint of its last write, on
every path out of it: a later sanitizing or clean overwrite counts, while a
write that only happens on some paths keeps the taint. An own-method call
between the writes still sees the earlier value.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Sanitized {
    private string $name;

    public function __construct(string $n) {
        $this->name = trim($n);
        $this->name = htmlspecialchars($this->name);
    }

    public function render(): void {
        echo $this->name;
    }
}

final class Unsanitized {
    private string $name;

    public function __construct(string $n) {
        $this->name = htmlspecialchars($n);
        $this->name = $n;
    }

    public function render(): void {
        echo $this->name;
    }
}

final class Replaced {
    private string $name;

    public function __construct(string $n) {
        $this->name = $n;
        $this->name = 'fixed';
    }

    public function render(): void {
        echo $this->name;
    }
}

final class SanitizedOnOnePath {
    private string $name;

    public function __construct(string $n, bool $escape) {
        $this->name = $n;
        if ($escape) {
            $this->name = htmlspecialchars($n);
        }
    }

    public function render(): void {
        echo $this->name;
    }
}

final class SanitizedOnEveryPath {
    private string $name;

    public function __construct(string $n, bool $upper) {
        if ($upper) {
            $this->name = htmlspecialchars(strtoupper($n));
        } else {
            $this->name = htmlspecialchars($n);
        }
    }

    public function render(): void {
        echo $this->name;
    }
}

final class Promoted {
    public function __construct(private string $name) {
        $this->name = htmlspecialchars($this->name);
    }

    public function render(): void {
        echo $this->name;
    }
}

final class PromotedKept {
    public function __construct(private string $name, private string $other) {
        $this->other = htmlspecialchars($this->other);
    }

    public function render(): void {
        echo $this->name;
    }

    public function renderOther(): void {
        echo $this->other;
    }
}

final class OverwrittenInLoop {
    private string $name;

    public function __construct(string $n, int $times) {
        $this->name = $n;
        for ($i = 0; $i < $times; $i++) {
            $this->name = htmlspecialchars($this->name);
        }
    }

    public function render(): void {
        echo $this->name;
    }
}

final class EarlyReturn {
    private string $name;

    public function __construct(string $n, bool $raw) {
        $this->name = $n;
        if ($raw) {
            return;
        }
        $this->name = htmlspecialchars($n);
    }

    public function render(): void {
        echo $this->name;
    }
}

final class EarlyReturnAfterSanitizing {
    private string $name;

    public function __construct(string $n, bool $skip) {
        $this->name = $n;
        $this->name = htmlspecialchars($n);
        if ($skip) {
            return;
        }
        $this->name .= '!';
    }

    public function render(): void {
        echo $this->name;
    }
}

final class AppendedAfterSanitizing {
    private string $name;

    public function __construct(string $n, string $suffix) {
        $this->name = htmlspecialchars($n);
        $this->name .= $suffix;
    }

    public function render(): void {
        echo $this->name;
    }
}

final class Conditional {
    private string $name = '';

    public function __construct(string $n, bool $set) {
        if ($set) {
            $this->name = $n;
        }
    }

    public function render(): void {
        echo $this->name;
    }
}

final class Setter {
    private string $name = '';

    public function set(string $n): void {
        $this->name = $n;
        $this->name = htmlspecialchars($this->name);
    }

    public function render(): void {
        echo $this->name;
    }
}

final class RawSetter {
    private string $name = '';

    public function set(string $n): void {
        $this->name = $n;
    }

    public function render(): void {
        echo $this->name;
    }
}

final class FlushedThenReset {
    private string $buffer = '';

    public function __construct(string $in) {
        $this->buffer = $in;
        $this->flush();
        $this->buffer = '';
    }

    public function flush(): void {
        echo $this->buffer;
    }
}

final class FlushedThroughStatic {
    private string $buffer = '';

    public function __construct(string $in) {
        $this->buffer = $in;
        self::dump($this);
        $this->buffer = '';
    }

    public static function dump(self $self): void {
        echo $self->buffer;
    }

    public function flush(): void {
        echo $this->buffer;
    }
}

final class ResetAfterUnrelatedCall {
    private string $buffer = '';

    public function __construct(string $in) {
        $this->buffer = $in;
        strlen($in);
        $this->buffer = '';
    }

    public function flush(): void {
        echo $this->buffer;
    }
}

final class Counter {
    private int $count = 0;

    public function __construct(string $n) {
        $this->count = (int) $n;
    }

    public function render(): void {
        echo $this->count;
    }
}

function build(): void {
    $a = new Sanitized($_GET['q']);
    $a->render();

    $b = new Unsanitized($_GET['q']);
    $b->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $c = new Replaced($_GET['q']);
    $c->render();

    $d = new SanitizedOnOnePath($_GET['q'], true);
    $d->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $e = new SanitizedOnEveryPath($_GET['q'], true);
    $e->render();

    $f = new Promoted($_GET['q']);
    $f->render();

    $g = new PromotedKept($_GET['q'], $_GET['r']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $g->renderOther();

    $h = new OverwrittenInLoop($_GET['q'], 2);
    $h->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $i = new EarlyReturn($_GET['q'], true);
    $i->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $j = new EarlyReturnAfterSanitizing($_GET['q'], true);
    $j->render();

    $k = new AppendedAfterSanitizing($_GET['q'], $_GET['r']);
    $k->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $l = new Conditional($_GET['q'], true);
    $l->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $m = new Counter($_GET['q']);
    $m->render();
}

function flushed_then_reset(): void {
    $a = new FlushedThenReset($_GET['q']);
    $a->flush();
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $b = new FlushedThroughStatic($_GET['q']);
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $b->flush();
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $c = new ResetAfterUnrelatedCall($_GET['q']);
    $c->flush();
}
