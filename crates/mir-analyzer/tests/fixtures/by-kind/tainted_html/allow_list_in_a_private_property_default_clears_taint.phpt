===description===
A strict check against a private property's literal-list default clears the value while nothing
in the class can change the property.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Fixed {
    private array $allowed = ['a', 'b'];
    private static array $modes = ['x', 'y'];

    public function instance(string $x): void {
        if (in_array($x, $this->allowed, true)) {
            echo $x;
        }
    }

    public function static(string $x): void {
        if (in_array($x, self::$modes, true)) {
            echo $x;
        }
    }
}

class Written {
    private array $allowed = ['a', 'b'];

    public function add(string $v): void {
        $this->allowed[] = $v;
    }

    public function check(string $x): void {
        if (in_array($x, $this->allowed, true)) {
            echo $x;
        }
    }
}

class ByRef {
    private array $allowed = ['a', 'b'];

    public function sortIt(): void {
        sort($this->allowed);
    }

    public function check(string $x): void {
        if (in_array($x, $this->allowed, true)) {
            echo $x;
        }
    }
}

class Visible {
    protected array $allowed = ['a', 'b'];

    public function check(string $x): void {
        if (in_array($x, $this->allowed, true)) {
            echo $x;
        }
    }
}

function run(): void {
    (new Fixed())->instance($_GET['x']);
    (new Fixed())->static($_GET['x']);
    (new Written())->check($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    (new ByRef())->check($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    (new Visible())->check($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
