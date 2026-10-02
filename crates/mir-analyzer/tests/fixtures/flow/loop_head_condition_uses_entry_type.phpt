===description===
A `while` condition is judged on the union of the loop-entry and back-edge
types, so a back-edge that proves the variable non-null does not make the
head comparison always true.
===file===
<?php
class Node {
    public ?Node $parent = null;
    public function up(): ?Node { return $this->parent; }
}

function walk(?Node $start): void {
    $node = $start;
    while ($node !== null) {
        $node = $node->parent;
    }
    /** @mir-check $node is null */
    echo 1;
}

function nullGuardBeforeAdvance(?Node $start): void {
    $n = $start;
    while ($n !== null) {
        $n = $n->parent;
        if ($n === null) {
            break;
        }
    }
}

function nestedLoopBreak(?Node $start): void {
    $n = $start;
    while ($n !== null) {
        foreach ([1] as $_) {
            $n = $n->parent;
            if ($n === null) {
                break 2;
            }
        }
    }
}

function lookAheadCondition(?Node $start): void {
    $n = $start;
    while ($n !== null && $n->parent !== null) {
        $n = $n->parent;
    }
}

function methodAdvance(?Node $start): void {
    $n = $start;
    while ($n !== null) {
        $n = $n->up();
    }
}

function genuinelyImpossibleInBody(?Node $start): void {
    $n = $start;
    while ($n !== null) {
        if ($n === null) {
//          ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'Node' and 'null' is always false — these types can never be identical
            echo 'dead';
        }
        $n = $n->parent;
    }
}

function headNullableAfterAdvance(): void {
    $n = new Node();
    while ($n !== null) {
        $n = $n->parent;
    }
}
function unchangedConstantHead(): void {
    $n = new Node();
    while ($n !== null) {
//         ^^^^^^^^^^^ ImpossibleIdenticalComparison: '!==' between 'Node' and 'null' is always true — these types can never be identical
        echo 1;
        if (rand(0, 1)) {
            break;
        }
    }
}
===expect===
