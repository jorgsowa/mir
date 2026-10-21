===description===
A property the method reads only through a method it calls on `$this`, `self::`,
`static::` or `parent::` still carries the object's taint to that method's sink
or return value. Clean, escaped or overwritten properties stay unreported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Page {
    protected string $title = '';
    protected string $note = 'fixed';

    public function __construct(string $title) {
        $this->title = $title;
    }

    protected function header(): void {
        echo $this->title;
    }

    protected function title(): string {
        return $this->title;
    }

    protected function note(): void {
        echo $this->note;
    }

    protected function show(string $s): void {
        echo $s;
    }

    public function viaThis(): void {
        $this->header();
    }

    public function viaSelf(): void {
        self::header();
    }

    public function viaStatic(): void {
        static::header();
    }

    public function viaThisClass(): void {
        $this::header();
    }

    public function viaReturn(): void {
        echo $this->title();
    }

    public function viaTwoHops(): void {
        $this->viaThis();
    }

    public function viaCleanNote(): void {
        $this->note();
    }

    public function viaArgument(string $s): void {
        $this->show($s);
    }
}

class Section extends Page {
    public function viaParent(): void {
        parent::header();
    }

    public function viaParentReturn(): void {
        echo parent::title();
    }

    public function viaParentTwoHops(): void {
        parent::viaThis();
    }
}

class Mutual {
    public string $v = '';

    public function __construct(string $v) {
        $this->v = $v;
    }

    public function ping(int $n): void {
        if ($n > 0) {
            $this->pong($n - 1);
        }
        echo $this->v;
    }

    public function pong(int $n): void {
        $this->ping($n);
    }
}

function direct(): void {
    $p = new Page($_GET['t']);
    $p->viaThis();
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function through_self(): void {
    $p = new Page($_GET['t']);
    $p->viaSelf();
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function through_static(): void {
    $p = new Page($_GET['t']);
    $p->viaStatic();
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function through_this_class(): void {
    $p = new Page($_GET['t']);
    $p->viaThisClass();
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function through_return(): void {
    $p = new Page($_GET['t']);
    $p->viaReturn();
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function two_hops(): void {
    $p = new Page($_GET['t']);
    $p->viaTwoHops();
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function from_subclass(): void {
    $s = new Section($_GET['t']);
    $s->viaParent();
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $s->viaParentReturn();
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $s->viaParentTwoHops();
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function mutual_recursion(): void {
    $m = new Mutual($_GET['t']);
    $m->pong(3);
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function argument_still_flows(): void {
    $p = new Page('clean');
    $p->viaArgument($_GET['t']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function clean_page(): void {
    $p = new Page('clean');
    $p->viaThis();
    $p->viaSelf();
    $p->viaReturn();
    $p->viaTwoHops();
    $s = new Section('clean');
    $s->viaParent();
    $s->viaParentReturn();
    $s->viaParentTwoHops();
    $m = new Mutual('clean');
    $m->pong(3);
}

function escaped_page(): void {
    $p = new Page(htmlspecialchars($_GET['t']));
    $p->viaThis();
    $p->viaSelf();
    $p->viaReturn();
    $s = new Section(htmlspecialchars($_GET['t']));
    $s->viaParent();
    $s->viaParentReturn();
}

function unread_property_stays_clean(): void {
    $p = new Page('clean');
    $p->viaCleanNote();
}

function tainted_title_does_not_taint_note(): void {
    $p = new Page($_GET['t']);
    $p->viaCleanNote();
}

function overwritten_before_call(): void {
    $p = new Page($_GET['t']);
    $p->viaCleanNote();
    $p = new Page('clean');
    $p->viaThis();
    $p->viaReturn();
}
