===description===
`@param-out static` on an instance method must resolve to the receiver's
concrete class, the same way `@return static` already does. The out-write-back
loop only substituted templates, never `TSelf`/`TStaticObject`, so the bare
`static` atom leaked to the caller unresolved.
===config===
suppress=UnusedVariable
===file===
<?php
class NodeA {
    /**
     * @param-out static $out
     */
    public function cloneInto(mixed &$out): void {
        $out = clone $this;
    }
}

class NodeB extends NodeA {}

$n = new NodeB();
$n->cloneInto($result);
/** @mir-check $result is NodeB */
$_ = $result;
===expect===
