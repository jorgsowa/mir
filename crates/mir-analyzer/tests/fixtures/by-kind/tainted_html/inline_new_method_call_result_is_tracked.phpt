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
}

function parenthesized() {
    echo ((new Source()))->fromRequest();
}

function nullsafe() {
    echo (new Source)?->fromRequest();
}

function sanitized() {
    echo (new Source)->escaped();
}

function constant() {
    echo (new Source)->literal();
}

function argumentTainted() {
    echo (new Source)->echoing($_GET['a']);
}

function argumentClean() {
    echo (new Source)->echoing('literal');
}

function viaOwnMethod() {
    echo (new Source)->chained();
}

function interpolated() {
    echo "<b>" . (new Source)->fromRequest() . "</b>";
}

function unknownClass() {
    echo (new Missing)->fromRequest();
}

function dynamicClass(string $class) {
    echo (new $class)->fromRequest();
}
===expect===
TaintedHtml@28:4-28:37: Tainted HTML output — possible XSS
TaintedHtml@32:4-32:41: Tainted HTML output — possible XSS
TaintedHtml@36:4-36:38: Tainted HTML output — possible XSS
TaintedHtml@48:4-48:43: Tainted HTML output — possible XSS
TaintedHtml@56:4-56:33: Tainted HTML output — possible XSS
TaintedHtml@60:4-60:54: Tainted HTML output — possible XSS
UndefinedClass@64:14-64:21: Class Missing does not exist
