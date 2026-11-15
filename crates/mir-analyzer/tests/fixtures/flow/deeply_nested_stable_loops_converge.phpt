===description===
Ten nested loops whose variables stop changing converge without widening or spurious issues.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foreachStable(array $xs): void {
    $s = 0;
    foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
        foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
            foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
                foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
                    foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
                        foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
                            foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
                                foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
                                    foreach ($xs as $x) { $s = rand(0, 1) ? 1 : $s;
                                        foreach ($xs as $x) { if (rand(0, 1)) { continue; } $s = rand(0, 1) ? 1 : $s;
    }}}}}}}}}}
    /** @mir-check $s is int */
    $s;
}

function whileStable(): void {
    $s = 0;
    while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
        while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
            while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
                while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
                    while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
                        while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
                            while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
                                while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
                                    while (rand(0, 1)) { $s = rand(0, 1) ? 1 : $s;
                                        while (rand(0, 1)) { if (rand(0, 1)) { continue; } $s = rand(0, 1) ? 1 : $s;
    }}}}}}}}}}
    /** @mir-check $s is int */
    $s;
}

function forStable(): void {
    $s = 0;
    for ($i0 = 0; $i0 < 10; $i0++) { $s = rand(0, 1) ? 1 : $s;
        for ($i1 = 0; $i1 < 10; $i1++) { $s = rand(0, 1) ? 1 : $s;
            for ($i2 = 0; $i2 < 10; $i2++) { $s = rand(0, 1) ? 1 : $s;
                for ($i3 = 0; $i3 < 10; $i3++) { $s = rand(0, 1) ? 1 : $s;
                    for ($i4 = 0; $i4 < 10; $i4++) { $s = rand(0, 1) ? 1 : $s;
                        for ($i5 = 0; $i5 < 10; $i5++) { $s = rand(0, 1) ? 1 : $s;
                            for ($i6 = 0; $i6 < 10; $i6++) { $s = rand(0, 1) ? 1 : $s;
                                for ($i7 = 0; $i7 < 10; $i7++) { $s = rand(0, 1) ? 1 : $s;
                                    for ($i8 = 0; $i8 < 10; $i8++) { $s = rand(0, 1) ? 1 : $s;
                                        for ($i9 = 0; $i9 < 10; $i9++) { if (rand(0, 1)) { continue; } $s = rand(0, 1) ? 1 : $s;
    }}}}}}}}}}
    /** @mir-check $s is int */
    $s;
}

function dowhileStable(): void {
    $s = 0;
    do { $s = rand(0, 1) ? 1 : $s;
        do { $s = rand(0, 1) ? 1 : $s;
            do { $s = rand(0, 1) ? 1 : $s;
                do { $s = rand(0, 1) ? 1 : $s;
                    do { $s = rand(0, 1) ? 1 : $s;
                        do { $s = rand(0, 1) ? 1 : $s;
                            do { $s = rand(0, 1) ? 1 : $s;
                                do { $s = rand(0, 1) ? 1 : $s;
                                    do { $s = rand(0, 1) ? 1 : $s;
                                        do { if (rand(0, 1)) { continue; } $s = rand(0, 1) ? 1 : $s;
    } while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));} while (rand(0, 1));
    /** @mir-check $s is int */
    $s;
}
