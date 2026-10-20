===description===
The property taint a constructor leaves on an object belongs to the object a
variable holds: rebinding the variable (assignment, wither, foreach, list
destructuring, unset) drops it, while copying or cloning the variable carries it.
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
class Greeting {
    public function __construct(private string $name) {}

    public function withName(string $name): static {
        return new static($name);
    }

    public function render(): void {
        echo '<p>Hello ' . $this->name . '</p>';
    }
}

function reassigned_to_clean_object(): void {
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $g = new Greeting('world');
    $g->render();
}

function reassigned_to_tainted_object(): void {
    $g = new Greeting('world');
    $g->render();
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function wither_result_replaces_object(): void {
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $g = $g->withName('world');
    $g->render();
}

function reassigned_to_unknown_value(Greeting $other): void {
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $g = $other;
    $g->render();
}

function rebound_by_foreach(array $list): void {
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    foreach ($list as $g) {
        $g->render();
    }
}

function rebound_by_foreach_key_and_value(array $list): void {
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    foreach ($list as $g => $h) {
        $h->render();
        echo $g;
    }
}

function unset_then_foreach(array $list): void {
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    unset($g);
    foreach ($list as $g) {
        $g->render();
    }
}

function rebound_by_destructuring(array $pair): void {
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    [$g] = $pair;
    $g->render();
}

function rebound_by_keyed_destructuring(array $pair): void {
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    ['a' => $g] = $pair;
    $g->render();
}

function self_assignment_keeps_taint(): void {
    $g = new Greeting($_GET['q']);
    $g = $g;
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function copy_carries_taint(): void {
    $g = new Greeting($_GET['q']);
    $copy = $g;
    $copy->render();
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function clone_carries_taint(): void {
    $g = new Greeting($_GET['q']);
    $copy = clone $g;
    $copy->render();
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $g = clone $g;
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function copy_over_tainted_object_takes_the_copy(): void {
    $clean = new Greeting('world');
    $g = new Greeting($_GET['q']);
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $g = $clean;
    $g->render();
}

function branch_that_may_keep_tainted_object(bool $c): void {
    $g = new Greeting($_GET['q']);
    if ($c) {
        $g = new Greeting('world');
    }
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

/** @param Greeting[] $list */
function foreach_over_nothing_may_keep_tainted_object(array $list): void {
    $g = new Greeting($_GET['q']);
    foreach ($list as $g) {
    }
    $g->render();
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
