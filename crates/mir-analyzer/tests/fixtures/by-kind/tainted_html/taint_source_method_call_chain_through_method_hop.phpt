===description===
`resolve_chained_receiver_type` had arms for a property, nullsafe
property, and array-index hop in the middle of a chained receiver, but
none for an intermediate METHOD-CALL hop — `$http->params()->get('id')`
(a taint-source method reached through one more call than a bare
variable/property/array chain) fell through to `None`, so the chain
walk died at the intermediate call and the whole expression was
untainted.
===config===
suppress=UnusedParam,MissingConstructor,MixedArrayAccess,MixedReturnStatement
===file===
<?php
class Param {
    /** @taint-source */
    public function get(string $key): string {
        return $_GET[$key] ?? '';
    }
}

class Http {
    public function params(): Param {
        return new Param();
    }
}

function leak(): void {
    $http = new Http();
    echo $http->params()->get('id');
}
===expect===
TaintedHtml@17:4-17:36: Tainted HTML output — possible XSS
