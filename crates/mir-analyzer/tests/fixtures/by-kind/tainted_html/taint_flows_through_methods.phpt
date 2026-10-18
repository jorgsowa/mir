===description===
Instance, static, inherited, trait and constructor calls carry taint through
their bodies like functions do. A method without a body (interface or
abstract) has nothing to summarize.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <PropertyNotSetInConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Formatter {
    public function format(string $v): string;
}

trait Renders {
    public function show(string $v): void { echo $v; }
}

class Page implements Formatter {
    use Renders;

    public function format(string $v): string { return '<p>' . $v . '</p>'; }
    public function title(): string { return (string) $_GET['title']; }
    public static function query(): string { return (string) $_GET['q']; }
    public function safe(string $v): string { return htmlspecialchars($v); }

    public function renderTitle(): void {
        echo $this->format($this->title());
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo self::query();
//      ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $this->safe($this->title());
    }
}

class SubPage extends Page {
    public function renderParent(): void {
        echo parent::format($_GET['x']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

class Banner {
    public function __construct(string $text) { echo $text; }
}

function useThem(Page $page, Formatter $formatter): void {
    echo $page->format($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo Page::query();
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $page->show($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    new Banner($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $page->safe($_GET['x']);
    echo $formatter->format($_GET['x']);
}
