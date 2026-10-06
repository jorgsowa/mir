===description===
A write at the end of a loop body is live when the next iteration reads it before reassigning.
===config===
<mir>
  <issueHandlers>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Cand { public string $key = ''; }

function use_it(Cand $o): void { echo $o->key; }

/** @param list<Cand> $items @param list<Cand> $candidates */
function nested_foreach(array $items, array $candidates): void {
    $a = null;
    foreach ($items as $item) {
        foreach ($candidates as $c) {
            if ($c->key === $item->key) { $a = $c; break; }
        }
        if ($a) { use_it($a); }
        $a = null;
    }
}

function while_loop(): void {
    $prev = null;
    while (rand(0, 1)) {
        if ($prev !== null) { use_it($prev); }
        $prev = rand(0, 1) ? new Cand() : null;
    }
}

function for_loop(): void {
    $a = null;
    for ($i = 0; $i < 3; $i++) {
        if ($a) { use_it($a); }
        $a = $i ? new Cand() : null;
    }
}

function do_while(): void {
    $a = null;
    do {
        if ($a) { use_it($a); }
        $a = rand(0, 1) ? new Cand() : null;
    } while (rand(0, 1));
}

function reset_in_conditional_branch(bool $flag): void {
    $a = null;
    while (rand(0, 1)) {
        if ($a) { use_it($a); }
        if ($flag) { $a = null; } else { $a = new Cand(); }
    }
}
