===description===
A call through an interface or abstract method moves taint the way any of
the known implementations does: their returns, sinks and source reads are
merged. When every implementation sanitizes, nothing is reported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Renderer {
    public function render(string $html): void;
}

class PageRenderer implements Renderer {
    public function render(string $html): void {
        echo $html;
    }
}

interface Formatter {
    public function format(string $text): string;
}

class PlainFormatter implements Formatter {
    public function format(string $text): string {
        return trim($text);
    }
}

class HtmlFormatter implements Formatter {
    public function format(string $text): string {
        return htmlspecialchars($text);
    }
}

interface Escaper {
    public function escape(string $text): string;
}

class HtmlEscaper implements Escaper {
    public function escape(string $text): string {
        return htmlspecialchars($text);
    }
}

abstract class Input {
    abstract public function value(): string;
}

class QueryInput extends Input {
    public function value(): string {
        return (string) $_GET['q'];
    }
}

function service(Renderer $renderer, Formatter $formatter, Escaper $escaper, Input $input): void {
    $renderer->render($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $renderer->render('<b>static</b>');
    echo $formatter->format($_GET['y']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $formatter->format('static');
    echo $escaper->escape($_GET['z']);
    echo $input->value();
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function show(Renderer $renderer, string $html): void {
    $renderer->render($html);
}

function read(Input $input): string {
    return $input->value();
}

function controller(Renderer $renderer, Input $input): void {
    show($renderer, $_GET['page']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo read($input);
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
