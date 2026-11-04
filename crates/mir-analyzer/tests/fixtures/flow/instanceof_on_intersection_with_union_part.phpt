===description===
`instanceof C` on `A&(B|C)` narrows to C instead of treating the union part as already a C
===file===
<?php
interface Ext {}
interface ConfExt {}
interface Conf {}
function needConf(Conf $c): Conf { return $c; }

function f(Ext $e): void {
    if ($e instanceof ConfExt || $e instanceof Conf) {
        if ($e instanceof Conf) {
            needConf($e);
        }
    }
}
