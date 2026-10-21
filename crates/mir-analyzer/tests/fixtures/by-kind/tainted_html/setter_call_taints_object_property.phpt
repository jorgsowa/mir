===description===
A method that stores its argument in a property taints that property of the
object it is called on, even when the object was built from clean data. A
later call that stores clean data, an escaped value or a different property
leaves the report off, and so does an object rebuilt or cloned beforehand.
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
class Base {
    public string $title = '';

    public function setTitle(string $title): void {
        $this->title = $title;
    }
}

class Box extends Base {
    public string $name = '';
    public string $other = '';

    public function __construct(string $name) {
        $this->name = $name;
    }

    public function setName(string $name): void {
        $this->name = $name;
    }

    public function setEscaped(string $name): void {
        $this->name = htmlspecialchars($name);
    }

    public function setBoth(string $name, string $other): void {
        $this->name = $name;
        $this->other = $other;
    }

    public function log(string $message): void {
        error_log($message);
    }

    public function name(): string {
        return $this->name;
    }
}

function direct(): void {
    $box = new Box('clean');
    $box->setName($_GET['a']);
    echo $box->name;
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function through_getter(): void {
    $box = new Box('clean');
    $box->setName($_GET['a']);
    echo $box->name();
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function inherited_setter(): void {
    $box = new Box('clean');
    $box->setTitle($_GET['a']);
    echo $box->title;
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $box->name;
}

function only_the_stored_argument(): void {
    $box = new Box('clean');
    $box->setBoth('fixed', $_GET['a']);
    echo $box->name;
    echo $box->other;
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function escaped_value(): void {
    $box = new Box('clean');
    $box->setEscaped($_GET['a']);
    echo $box->name;
}

function other_method_does_not_store(): void {
    $box = new Box('clean');
    $box->log($_GET['a']);
    echo $box->name;
}

function later_clean_call_overwrites(): void {
    $box = new Box('clean');
    $box->setName($_GET['a']);
    $box->setName('fixed');
    echo $box->name;
}

function later_escaped_call_overwrites(): void {
    $box = new Box($_GET['a']);
    $box->setEscaped($_GET['b']);
    echo $box->name;
}

function branch_taints_after_merge(bool $flag): void {
    $box = new Box('clean');
    if ($flag) {
        $box->setName($_GET['a']);
    }
    echo $box->name;
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function both_branches_clean(bool $flag): void {
    $box = new Box('clean');
    if ($flag) {
        $box->setName('a');
    } else {
        $box->setName('b');
    }
    echo $box->name;
}

function loop_taints(array $items): void {
    $box = new Box('clean');
    foreach ($items as $item) {
        $box->setName($_GET[(string) $item]);
    }
    echo $box->name;
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function rebuilt_object(): void {
    $box = new Box('clean');
    $box->setName($_GET['a']);
    $box = new Box('fresh');
    echo $box->name;
}

function clone_before_the_call(): void {
    $box = new Box('clean');
    $copy = clone $box;
    $box->setName($_GET['a']);
    echo $copy->name;
    echo $box->name;
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function clone_after_the_call(): void {
    $box = new Box('clean');
    $box->setName($_GET['a']);
    $copy = clone $box;
    echo $copy->name;
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function separate_objects(): void {
    $one = new Box('clean');
    $two = new Box('clean');
    $one->setName($_GET['a']);
    echo $two->name;
    echo $one->name;
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function clean_argument(): void {
    $box = new Box('clean');
    $box->setName('fixed');
    echo $box->name;
}
