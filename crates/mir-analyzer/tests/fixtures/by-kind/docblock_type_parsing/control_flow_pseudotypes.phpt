===description===
`void`, `never` and its aliases (`never-return`, `never-returns`,
`no-return`) parse to their canonical atom. `empty` (Psalm's falsy
pseudo-type) parses without error; its exact expansion is intentionally not
pinned down here — see the description in
associative_array_and_arraylike_object.phpt for why composite expansions are
risky to hand-encode.
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
function check_void($x) {
    /**
     * @var void $x
     * @mir-check $x is void
     */
    var_dump($x);
}

function check_never($x) {
    /**
     * @var never $x
     * @mir-check $x is never
     */
    var_dump($x);
}

function check_never_return($x) {
    /**
     * @var never-return $x
     * @mir-check $x is never
     */
    var_dump($x);
}

function check_never_returns($x) {
    /**
     * @var never-returns $x
     * @mir-check $x is never
     */
    var_dump($x);
}

function check_no_return($x) {
    /**
     * @var no-return $x
     * @mir-check $x is never
     */
    var_dump($x);
}

function check_empty($x) {
    /**
     * @var empty $x
     */
    var_dump($x);
}
===expect===
