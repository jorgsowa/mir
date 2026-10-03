===description===
`for` condition on a variable reassigned by the update expression is not
always-true: the pre-loop state ignores the update (`$c = $c->getPrevious()`).
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function depth(\Throwable $e): int {
    $n = 0;
    for ($c = $e; $c !== null; $c = $c->getPrevious()) {
        $n++;
    }
    return $n;
}

class Node {
    public ?Node $next = null;
}
function length(Node $head): int {
    $n = 0;
    for ($cur = $head, $i = 0; $cur !== null; $cur = $cur->next, $i++) {
        $n = $i;
    }
    return $n;
}

function last(Node $head): Node {
    for ($cur = $head; $cur->next !== null; $cur = $cur->next) {
    }
    /** @mir-check $cur is Node */
    return $cur;
}
===expect===
