===description===
@taint-source on a method call reached through a chained receiver with an
array-index hop in the middle ($this->repos['main']->getParam()) fell
through to the catch-all untainted case -- resolve_chained_receiver_type
had no ArrayAccess arm, unlike its sibling root_receiver_var, so the
chain broke off with None before ever reaching the taint-source check.
===config===
suppress=UnusedParam,MissingConstructor,MixedReturnStatement,MixedArrayAccess,MissingPropertyType
===file===
<?php
class Request {
    /** @taint-source */
    public function getParam(string $name): string {
        return $_GET[$name] ?? '';
    }
}

class Handler {
    /** @var array<string, Request> */
    public array $repos;

    public function handle(): void {
        echo $this->repos['main']->getParam('x');
    }
}
===expect===
TaintedHtml@14:8-14:49: Tainted HTML output — possible XSS
