===description===
`static` nested in a return (`list<static>`) called on an `A&B` receiver is the whole intersection
===file===
<?php
interface Pool {}
interface Tree {
    /** @return list<static> */
    public function children(): array;
}

function f(Pool $p): void {
    if ($p instanceof Tree) {
        $c = $p->children();
        /** @mir-check $c is list<Pool&Tree> */
        echo count($c);
    }
}
