===description===
`callable`/`Closure` signature syntax, bare purity-qualified keywords
(`pure-callable`, `pure-closure`), and a signature with no explicit return
type (defaults to `mixed`).
===config===
suppress=MissingReturnType,MissingParamType,ForbiddenCode
===file===
<?php
function check_bare_callable($x) {
    /**
     * @var callable $x
     * @mir-check $x is callable
     */
    var_dump($x);
}

function check_pure_callable_bare($x) {
    /**
     * @var pure-callable $x
     * @mir-check $x is callable
     */
    var_dump($x);
}

function check_callable_with_signature($x) {
    /**
     * @var callable(int, string): bool $x
     * @mir-check $x is callable(int, string): bool
     */
    var_dump($x);
}

function check_callable_no_return($x) {
    /**
     * @var callable(int) $x
     * @mir-check $x is callable(int): mixed
     */
    var_dump($x);
}

function check_closure_with_signature($x) {
    /**
     * @var Closure(int): string $x
     * @mir-check $x is Closure(int): string
     */
    var_dump($x);
}

function check_pure_closure_bare($x) {
    /**
     * @var pure-closure $x
     * @mir-check $x is Closure
     */
    var_dump($x);
}
===expect===
