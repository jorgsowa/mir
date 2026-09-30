===description===
@taint-source on a method call only recognized a bare-variable receiver
($req->getParam()) -- a chained property receiver ($this->req->getParam())
fell through to the catch-all untainted case, even though the method
itself is annotated.
===config===
suppress=UnusedParam,MissingConstructor,MixedReturnStatement,MixedArrayAccess
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
