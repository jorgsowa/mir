===description===
A variable wrapped in one more array on every pass of nested loops widens to a generic list instead of deepening forever, also when the inner loop continues or breaks.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function nestedForeachContinue(array $xs): void {
    $s = 0;
    foreach ($xs as $x) { $s = [$s];
        foreach ($xs as $x) { $s = [$s];
            foreach ($xs as $x) { $s = [$s];
                foreach ($xs as $x) { if ($x) { continue; } $s = [$s]; }
    }}}
    /** @mir-check $s is non-empty-list<non-empty-list|int>|int */
    $s;
}

/** @param list<string> $xs */
function linkedNodes(array $xs): void {
    $node = null;
    foreach ($xs as $x) {
        $node = ['value' => $x, 'next' => $node];
    }
    /** @mir-check $node is non-empty-array<string, string|non-empty-array<string, mixed>|null>|null */
    $node;
}

function foreachOnce(array $xs): void {
    $s = 0;
    foreach ($xs as $x) { $s = [$s];
    }
    /** @mir-check $s is non-empty-list<non-empty-list|int>|int */
    $s;
}

function whileOnce(): void {
    $s = 0;
    while (rand(0, 1)) { $s = [$s];
    }
    /** @mir-check $s is non-empty-list<non-empty-list|int>|int */
    $s;
}

function forOnce(): void {
    $s = 0;
    for ($i0 = 0; $i0 < 10; $i0++) { $s = [$s];
    }
    /** @mir-check $s is non-empty-list<non-empty-list|int> */
    $s;
}

function dowhileOnce(): void {
    $s = 0;
    do { $s = [$s];
    } while (rand(0, 1));
    /** @mir-check $s is non-empty-list<non-empty-list|int> */
    $s;
}


function foreachContinue(array $xs): void {
    $s = 0;
    foreach ($xs as $x) { $s = [$s];
        foreach ($xs as $x) { $s = [$s];
            foreach ($xs as $x) { $s = [$s];
                foreach ($xs as $x) { $s = [$s];
                    foreach ($xs as $x) { $s = [$s];
                        foreach ($xs as $x) { $s = [$s];
                            foreach ($xs as $x) { $s = [$s];
                                foreach ($xs as $x) { $s = [$s];
                                    foreach ($xs as $x) { $s = [$s];
                                        foreach ($xs as $x) { if (rand(0, 1)) { continue; } $s = [$s];
    }}}}}}}}}}
    /** @mir-check $s is non-empty-list<non-empty-list|int>|int */
    $s;
}

function foreachBreak(array $xs): void {
    $s = 0;
    foreach ($xs as $x) { $s = [$s];
        foreach ($xs as $x) { $s = [$s];
            foreach ($xs as $x) { $s = [$s];
                foreach ($xs as $x) { $s = [$s];
                    foreach ($xs as $x) { $s = [$s];
                        foreach ($xs as $x) { $s = [$s];
                            foreach ($xs as $x) { $s = [$s];
                                foreach ($xs as $x) { if (rand(0, 1)) { break; } $s = [$s];
    }}}}}}}}
    /** @mir-check $s is non-empty-list<non-empty-list|int>|int */
    $s;
}

function whileContinue(): void {
    $s = 0;
    while (rand(0, 1)) { $s = [$s];
        while (rand(0, 1)) { $s = [$s];
            while (rand(0, 1)) { $s = [$s];
                while (rand(0, 1)) { $s = [$s];
                    while (rand(0, 1)) { $s = [$s];
                        while (rand(0, 1)) { $s = [$s];
                            while (rand(0, 1)) { $s = [$s];
                                while (rand(0, 1)) { $s = [$s];
                                    while (rand(0, 1)) { $s = [$s];
                                        while (rand(0, 1)) { if (rand(0, 1)) { continue; } $s = [$s];
    }}}}}}}}}}
    /** @mir-check $s is non-empty-list<non-empty-list|int>|int */
    $s;
}

function whileBreak(): void {
    $s = 0;
    while (rand(0, 1)) { $s = [$s];
        while (rand(0, 1)) { $s = [$s];
            while (rand(0, 1)) { $s = [$s];
                while (rand(0, 1)) { $s = [$s];
                    while (rand(0, 1)) { $s = [$s];
                        while (rand(0, 1)) { $s = [$s];
                            while (rand(0, 1)) { $s = [$s];
                                while (rand(0, 1)) { if (rand(0, 1)) { break; } $s = [$s];
    }}}}}}}}
    /** @mir-check $s is non-empty-list<non-empty-list|int>|int */
    $s;
}

function forContinue(): void {
    $s = 0;
    for ($i0 = 0; $i0 < 10; $i0++) { $s = [$s];
        for ($i1 = 0; $i1 < 10; $i1++) { $s = [$s];
            for ($i2 = 0; $i2 < 10; $i2++) { $s = [$s];
                for ($i3 = 0; $i3 < 10; $i3++) { $s = [$s];
                    for ($i4 = 0; $i4 < 10; $i4++) { $s = [$s];
                        for ($i5 = 0; $i5 < 10; $i5++) { $s = [$s];
                            for ($i6 = 0; $i6 < 10; $i6++) { $s = [$s];
                                for ($i7 = 0; $i7 < 10; $i7++) { $s = [$s];
                                    for ($i8 = 0; $i8 < 10; $i8++) { $s = [$s];
                                        for ($i9 = 0; $i9 < 10; $i9++) { if (rand(0, 1)) { continue; } $s = [$s];
    }}}}}}}}}}
    /** @mir-check $s is non-empty-list<non-empty-list> */
    $s;
}

function forBreak(): void {
    $s = 0;
    for ($i0 = 0; $i0 < 10; $i0++) { $s = [$s];
        for ($i1 = 0; $i1 < 10; $i1++) { $s = [$s];
            for ($i2 = 0; $i2 < 10; $i2++) { $s = [$s];
                for ($i3 = 0; $i3 < 10; $i3++) { $s = [$s];
                    for ($i4 = 0; $i4 < 10; $i4++) { $s = [$s];
                        for ($i5 = 0; $i5 < 10; $i5++) { $s = [$s];
                            for ($i6 = 0; $i6 < 10; $i6++) { $s = [$s];
                                for ($i7 = 0; $i7 < 10; $i7++) { if (rand(0, 1)) { break; } $s = [$s];
    }}}}}}}}
    /** @mir-check $s is non-empty-list<non-empty-list> */
    $s;
}

function dowhileContinue(): void {
    $s = 0;
    do { $s = [$s];
        do { $s = [$s];
            do { $s = [$s];
                do { $s = [$s];
                    do { $s = [$s];
                        do { $s = [$s];
                            do { $s = [$s];
                                do { $s = [$s];
                                    do { $s = [$s];
                                        do { if (rand(0, 1)) { continue; } $s = [$s];
    } while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));
    /** @mir-check $s is non-empty-list<non-empty-list> */
    $s;
}

function dowhileBreak(): void {
    $s = 0;
    do { $s = [$s];
        do { $s = [$s];
            do { $s = [$s];
                do { $s = [$s];
                    do { $s = [$s];
                        do { $s = [$s];
                            do { $s = [$s];
                                do { if (rand(0, 1)) { break; } $s = [$s];
    } while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));
    /** @mir-check $s is non-empty-list<non-empty-list> */
    $s;
}
