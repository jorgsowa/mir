===description===
`callable-array` and `callable-object` parse to their base type; `static-closure` and `static-pure-closure` to a closure taking any arguments.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function check_callable_array($x) {
    /**
     * @var callable-array $x
     * @mir-check $x is array<array-key, mixed>&callable
     */
    var_dump($x);
}

function check_callable_object($x) {
    /**
     * @var callable-object $x
     * @mir-check $x is object&callable
     */
    var_dump($x);
}

function check_static_closure($x) {
    /**
     * @var static-closure $x
     * @mir-check $x is Closure(mixed): mixed
     */
    var_dump($x);
}

function check_static_pure_closure($x) {
    /**
     * @var static-pure-closure $x
     * @mir-check $x is pure-Closure(mixed): mixed
     */
    var_dump($x);
}
