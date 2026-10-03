===description===
Psalm/PHPStan string refinement pseudo-types. mir does not model
case/falsiness precisely, so several of these are approximations (documented
in parser/docblock/types.rs) — the checks below assert the actual
approximation, not an idealized one.
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
function check_non_empty_string($x) {
    /**
     * @var non-empty-string $x
     * @mir-check $x is non-empty-string
     */
    var_dump($x);
}

function check_numeric_string($x) {
    /**
     * @var numeric-string $x
     * @mir-check $x is numeric-string
     */
    var_dump($x);
}

function check_truthy_string($x) {
    /**
     * @var truthy-string $x
     * @mir-check $x is non-empty-string
     */
    var_dump($x);
}

function check_non_falsy_string($x) {
    /**
     * @var non-falsy-string $x
     * @mir-check $x is non-empty-string
     */
    var_dump($x);
}

function check_lowercase_string($x) {
    /**
     * @var lowercase-string $x
     * @mir-check $x is string
     */
    var_dump($x);
}

function check_uppercase_string($x) {
    /**
     * @var uppercase-string $x
     * @mir-check $x is string
     */
    var_dump($x);
}

function check_non_empty_lowercase_string($x) {
    /**
     * @var non-empty-lowercase-string $x
     * @mir-check $x is string
     */
    var_dump($x);
}

function check_non_empty_uppercase_string($x) {
    /**
     * @var non-empty-uppercase-string $x
     * @mir-check $x is string
     */
    var_dump($x);
}

function check_literal_string($x) {
    /**
     * @var literal-string $x
     * @mir-check $x is string
     */
    var_dump($x);
}

function check_class_string($x) {
    /**
     * @var class-string $x
     * @mir-check $x is class-string
     */
    var_dump($x);
}

function check_callable_string($x) {
    /**
     * @var callable-string $x
     * @mir-check $x is callable-string
     */
    var_dump($x);
}

function check_interface_string($x) {
    /**
     * @var interface-string $x
     * @mir-check $x is interface-string
     */
    var_dump($x);
}

function check_trait_string($x) {
    /**
     * @var trait-string $x
     * @mir-check $x is trait-string
     */
    var_dump($x);
}

function check_enum_string($x) {
    /**
     * @var enum-string $x
     * @mir-check $x is enum-string
     */
    var_dump($x);
}
===expect===
