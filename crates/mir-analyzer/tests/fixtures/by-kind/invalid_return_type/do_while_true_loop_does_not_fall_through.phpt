===description===
`do { … } while (true)` only exits through `break`, so a function whose loop
returns and never breaks doesn't fall through to an implicit `void` return.
===file===
<?php
final class W {
    public function ok(): bool { return true; }
}

function retryUntilOk(): W {
    do {
        $w = new W();
        if (!$w->ok()) {
            continue;
        }
        return $w;
    } while (true);
}

function insideTry(): W {
    try {
        do {
            if (rand(0, 1)) {
                return new W();
            }
        } while (1);
    } finally {
        echo 1;
    }
}

function breaks(): W {
//                 ^ InvalidReturnType: Return type 'void' is not compatible with declared 'W'
    do {
        if (rand(0, 1)) {
            break;
        }
        if (rand(0, 1)) {
            return new W();
        }
    } while (true);
}

function afterLoop(): int {
    $i = 0;
    do {
        $i++;
        if ($i > 3) {
            return $i;
        }
    } while (true);
    return -1;
//  ^^^^^^^^^^ UnreachableCode: Unreachable code detected
}
