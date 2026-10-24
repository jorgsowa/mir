===description===
A method called straight on an inline `new` returns what its own body reads or
what the arguments carry, as it does on a stored object. Clean, sanitized and
void results stay unreported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Source {
    public function fromRequest() {
        return $_GET['a'];
    }

    public function escaped() {
        return htmlspecialchars($_GET['a']);
    }

    public function literal() {
        return 'literal';
    }

    public function echoing($value) {
        return $value;
    }

    public function chained() {
        return $this->fromRequest();
    }

    public function nothing(): void {
    }
}

function fromBody() {
    echo (new Source)->fromRequest();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function parenthesized() {
    echo ((new Source()))->fromRequest();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function nullsafe() {
    echo (new Source)?->fromRequest();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function sanitized() {
    echo (new Source)->escaped();
}

function constant() {
    echo (new Source)->literal();
}

function argumentTainted() {
    echo (new Source)->echoing($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function argumentClean() {
    echo (new Source)->echoing('literal');
}

function viaOwnMethod() {
    echo (new Source)->chained();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function interpolated() {
    echo "<b>" . (new Source)->fromRequest() . "</b>";
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function unknownClass() {
    echo (new Missing)->fromRequest();
//            ^^^^^^^ UndefinedClass: Class Missing does not exist
}

function dynamicClass(string $class) {
    echo (new $class)->fromRequest();
}
