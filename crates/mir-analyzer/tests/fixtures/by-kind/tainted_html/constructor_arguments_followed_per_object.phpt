===description===
An argument a constructor stores in a property, directly or through a
promoted parameter, taints that property of the new object: reading it, or
calling a method that echoes or returns it, is reported. Another object of
the same class built from clean data isn't.
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
class Greeting {
    public string $name;
    private string $escaped;

    public function __construct(string $name) {
        $this->name = $name;
        $this->escaped = htmlspecialchars($name);
    }

    public function render(): void {
        echo '<p>Hello ' . $this->name . '</p>';
    }

    public function renderEscaped(): void {
        echo $this->escaped;
    }

    public function name(): string {
        return $this->name;
    }
}

final class Banner {
    public function __construct(private string $text, private int $width = 80) {}

    public function show(): void {
        echo $this->text;
    }

    public function width(): int {
        return $this->width;
    }
}

function page(): void {
    $greeting = new Greeting($_GET['q']);
    $greeting->render();
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $greeting->renderEscaped();
    echo $greeting->name;
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $greeting->name();
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS

    $static = new Greeting('world');
    $static->render();
    echo $static->name();

    $banner = new Banner($_GET['b']);
    $banner->show();
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $banner->width();

    $greeting = new Greeting('again');
    $greeting->render();
}
