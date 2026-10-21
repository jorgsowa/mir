===description===
A property written through one variable is seen through every variable that
may hold the same object, whether the write is direct or a setter call. A
variable rebound to another object leaves the group, and a clone is separate.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Cell {
    public string $n = '';

    public function __construct(string $n) {
        $this->n = $n;
    }

    public function setN(string $n): void {
        $this->n = $n;
    }
}

function direct_write(): void {
    $o = new Cell('a');
    $t = $o;
    $t->n = $_GET['b'];
    echo $o->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function setter_write(): void {
    $o = new Cell('a');
    $t = $o;
    $t->setN($_GET['b']);
    echo $o->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function reference_write(): void {
    $o = new Cell('a');
    $t = &$o;
    $t->n = $_GET['b'];
    echo $o->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function chain_of_copies(): void {
    $o = new Cell('a');
    $t = $o;
    $u = $t;
    $u->n = $_GET['b'];
    echo $o->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $t->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function original_rebound_before_write(): void {
    $o = new Cell('a');
    $t = $o;
    $o = new Cell('b');
    $t->n = $_GET['b'];
    echo $o->n;
    echo $t->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function copy_rebound_before_write(): void {
    $o = new Cell('a');
    $t = $o;
    echo $t->n;
    $t = new Cell('b');
    $t->n = $_GET['b'];
    echo $o->n;
    echo $t->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function root_rebound_keeps_other_members(): void {
    $o = new Cell('a');
    $t = $o;
    $u = $t;
    $o = new Cell('b');
    $u->n = $_GET['b'];
    echo $t->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $o->n;
}

function rebound_root_does_not_join_old_group(): void {
    $o = new Cell('a');
    $t = $o;
    $o = new Cell('b');
    $x = $o;
    $x->n = $_GET['b'];
    echo $t->n;
    echo $o->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function clone_is_separate(): void {
    $o = new Cell('a');
    $t = clone $o;
    $t->n = $_GET['b'];
    echo $o->n;
}

function aliased_in_one_branch(bool $flag): void {
    $o = new Cell('a');
    $t = new Cell('b');
    if ($flag) {
        $t = $o;
    }
    $t->n = $_GET['b'];
    echo $o->n;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function clean_write_through_alias(): void {
    $o = new Cell('a');
    $t = $o;
    $t->n = 'clean';
    echo $o->n;
}

function two_separate_objects(): void {
    $o = new Cell('a');
    $p = new Cell('b');
    $p->n = $_GET['b'];
    echo $o->n;
}
