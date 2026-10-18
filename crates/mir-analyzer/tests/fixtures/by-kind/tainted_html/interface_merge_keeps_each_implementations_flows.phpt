===description===
A call through an interface or abstract method merges only the
implementations the receiver's type can hold, so a narrower sub-interface or
abstract subclass leaves its siblings out. Each implementation's property and
object-guard labels keep their own meaning in the merge, even when two
implementations number them the same or one adds a parameter where another
keeps a label.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Renderer {
    public function render(string $html): void;
}
interface SafeRenderer extends Renderer {}

class EscapingRenderer implements SafeRenderer {
    public function render(string $html): void {
        echo htmlspecialchars($html);
    }
}
class RawRenderer implements Renderer {
    public function render(string $html): void {
        echo $html;
    }
}

abstract class Field {
    abstract public function value(): string;
}
abstract class FixedField extends Field {}

class ConstantField extends FixedField {
    public function value(): string {
        return 'constant';
    }
}
class QueryField extends Field {
    public function value(): string {
        return (string) $_GET['q'];
    }
}

interface Printer {
    public function print(): void;
}
class TitlePrinter implements Printer {
    public function __construct(private string $title) {}
    public function print(): void {
        echo $this->title;
    }
}
class FooterPrinter implements Printer {
    public function __construct(private string $footer) {}
    public function print(): void {
        echo $this->footer;
    }
}

interface Banner {
    /** @taint-sink html $text */
    public function show(string $text): void;
}
class HeaderBanner implements Banner {
    public function __construct(private string $header) {}
    public function show(string $text): void {
        echo $this->header;
    }
}
class FooterBanner implements Banner {
    public function __construct(private string $footer) {}
    public function show(string $text): void {
        echo $this->footer;
    }
}

interface Runner {
    public function run(string $label): void;
}
class CommandRunner implements Runner {
    public function __construct(private string $command) {}
    public function run(string $label): void {
        system($this->command);
    }
}
class VerboseRunner implements Runner {
    public function run(string $label, string $detail = ''): void {
        echo $detail;
    }
}

interface Handler {
    public function handle(mixed $first, mixed $second): void;
}
class FirstHandler implements Handler {
    public function handle(mixed $first, mixed $second): void {
        if ($first instanceof Closure) {
            $first();
        }
    }
}
class SecondHandler implements Handler {
    public function handle(mixed $first, mixed $second): void {
        if ($second instanceof Closure) {
            $second();
        }
    }
}

function narrowerTypes(SafeRenderer $safe, Renderer $any, FixedField $fixed, Field $field): void {
    $safe->render($_GET['x']);
    $any->render($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $fixed->value();
    echo $field->value();
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function propertyLabels(): void {
    /** @var Printer $printer */
    $printer = new FooterPrinter((string) $_GET['footer']);
    $printer->print();
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    /** @var Runner $runner */
    $runner = new CommandRunner((string) $_GET['cmd']);
    $runner->run('label');
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    /** @var Banner $banner */
    $banner = new HeaderBanner('static');
    $banner->show((string) $_GET['text']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function objectGuards(Handler $handler): void {
    $handler->handle($_GET['a'], $_GET['b']);
    $handler->handle(json_decode((string) $_GET['a']), 'b');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
}
