===description===
@taint-source on a method call only recognized a bare-variable receiver
($req->getParam()) -- a chained property receiver ($this->req->getParam())
fell through to the catch-all untainted case, even though the method
itself is annotated.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Request {
    /** @taint-source */
    public function getParam(string $name): string {
        return $_GET[$name] ?? '';
    }
}

class Handler {
    public Request $req;

    public function handle(): void {
        echo $this->req->getParam('x');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
===expect===
