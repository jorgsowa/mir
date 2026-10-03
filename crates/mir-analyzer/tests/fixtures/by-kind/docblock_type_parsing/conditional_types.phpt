===description===
`($param is TypeName ? TrueType : FalseType)` conditional-type syntax,
including a nested conditional in the false branch (exercises
`find_is_marker_at_depth`'s depth tracking across the outer/inner
parens — no prior fixture covered nesting).
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function check_conditional($value) {
    /**
     * @var ($value is int ? string : bool) $x
     * @mir-check $x is ($value is int ? string : bool)
     */
    var_dump($x);
}

function check_nested_conditional($value) {
    /**
     * @var ($value is int ? string : ($value is string ? bool : float)) $x
     * @mir-check $x is ($value is int ? string : ($value is string ? bool : float))
     */
    var_dump($x);
}
===expect===
