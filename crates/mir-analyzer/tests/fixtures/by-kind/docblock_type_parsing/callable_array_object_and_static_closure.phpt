===description===
`callable-array`, `callable-object`, `static-closure` and `static-pure-closure` parse to their base type.
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
     * @mir-check $x is array<array-key, mixed>
     */
    var_dump($x);
}

function check_callable_object($x) {
    /**
     * @var callable-object $x
     * @mir-check $x is object
     */
    var_dump($x);
}

function check_static_closure($x) {
    /**
     * @var static-closure $x
     * @mir-check $x is Closure
     */
    var_dump($x);
}

function check_static_pure_closure($x) {
    /**
     * @var static-pure-closure $x
     * @mir-check $x is Closure
     */
    var_dump($x);
}
