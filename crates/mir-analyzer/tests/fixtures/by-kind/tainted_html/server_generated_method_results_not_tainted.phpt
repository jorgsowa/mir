===description===
A method whose result is built from the arguments or server-side state does not
carry the receiver's tainted data: date rendering and the generated name or path
of an uploaded file. Arguments still flow, and other classes' methods of the same
names stay tainted.
===config===

<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <InvalidReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Symfony\Component\HttpFoundation\File {
    class UploadedFile extends \SplFileInfo {
        public function __construct(string $path, private string $name) { parent::__construct($path); }
        public function hashName(): string { return md5($this->name); }
        public function store(string $dir): string { return $dir . '/x'; }
        public function storeAs(string $dir, string $name): string { return $dir . '/' . $name; }
    }
}

namespace Illuminate\Http {
    class UploadedFile extends \Symfony\Component\HttpFoundation\File\UploadedFile {}
}

namespace App {
    use Illuminate\Http\UploadedFile;

    class Label {
        public function __construct(private string $text) {}
        public function format(string $f): string { return $this->text; }
        public function store(string $dir): string { return $this->text; }
    }

    class Clock extends \DateTimeImmutable {}

    function dates(\DateTimeInterface $d): void {
        echo date_create($_GET['d'])->format('Y-m-d');
        echo (new \DateTimeImmutable($_GET['d']))->format('Y');
        echo (new Clock($_GET['d']))->format('Y');
        echo date_create($_GET['d'])?->format('Y');
        $parsed = new \DateTime($_GET['d']);
        echo $parsed->format('Y');
        echo $parsed->getTimezone()->getName();
        echo $d->format('Y');
        echo (new \DateInterval($_GET['i']))->format('%d');
    }

    function dateArguments(): void {
        echo date_create('now')->format($_GET['f']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo (new \DateTime())->format($_GET['f']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function otherClasses(): void {
        $label = new Label($_GET['t']);
        echo $label->format('x');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
        echo $label->store('dir');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }

    function uploads(): void {
        $u = new UploadedFile($_GET['p'], $_GET['n']);
        echo $u->hashName();
        echo $u->store('dir');
        echo $u->getRealPath();
        echo $u->getPathname();
        echo $u->storeAs('dir', $_GET['n']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
