===description===
`resolve_chained_receiver_type` had arms for a property, nullsafe property,
array-index, and instance-method-call hop in the middle of a chained
receiver, but none for a STATIC-PROPERTY hop — `self::$param->get('id')`
(a taint-source method reached through a static property one hop up) fell
through to `None`, so the chain walk died at the static-property hop and the
whole expression was untainted.
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
    public static Param $param;

    public function leak(): void {
        self::$param = new Param();
        echo self::$param->get('id');
    }
}
===expect===
TaintedHtml@14:8-14:37: Tainted HTML output — possible XSS
