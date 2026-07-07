===description===
`@param-out static` on a static-call-syntax method (`Foo::make(...)`) must
also resolve to the receiver's concrete class. static_call.rs's out-write-back
loop only substituted the merged template bindings, never `TSelf`/
`TStaticObject`, so the bare `static` atom leaked to the caller unresolved.
===config===
suppress=UnusedVariable
===file===
<?php
class Factory {
    /**
     * @param-out static $out
     */
    public static function make(mixed &$out): void {
        $out = new static();
    }
}

class SubFactory extends Factory {}

SubFactory::make($result);
/** @mir-check $result is SubFactory */
$_ = $result;
===expect===
