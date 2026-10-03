===description===
Scalar pseudo-type aliases (`integer`, `double`, `boolean`, `array-key`,
`resource`) parse to their canonical atom. `@var` forcibly retypes the
variable, so `@mir-check` right after it reads back exactly what the
docblock type parser produced.
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
function check_integer_alias($x) {
    /**
     * @var integer $x
     * @mir-check $x is int
     */
    var_dump($x);
}

function check_double_alias($x) {
    /**
     * @var double $x
     * @mir-check $x is float
     */
    var_dump($x);
}

function check_boolean_alias($x) {
    /**
     * @var boolean $x
     * @mir-check $x is bool
     */
    var_dump($x);
}

function check_array_key($x) {
    /**
     * @var array-key $x
     * @mir-check $x is int|string
     */
    var_dump($x);
}

function check_resource($x) {
    /**
     * @var resource $x
     * @mir-check $x is mixed
     */
    var_dump($x);
}

function check_scalar($x) {
    /**
     * @var scalar $x
     * @mir-check $x is scalar
     */
    var_dump($x);
}

function check_numeric($x) {
    /**
     * @var numeric $x
     * @mir-check $x is numeric
     */
    var_dump($x);
}

function check_mixed($x) {
    /**
     * @var mixed $x
     * @mir-check $x is mixed
     */
    var_dump($x);
}

function check_object($x) {
    /**
     * @var object $x
     * @mir-check $x is object
     */
    var_dump($x);
}

function check_null($x) {
    /**
     * @var null $x
     * @mir-check $x is null
     */
    var_dump($x);
}

function check_true($x) {
    /**
     * @var true $x
     * @mir-check $x is true
     */
    var_dump($x);
}

function check_false($x) {
    /**
     * @var false $x
     * @mir-check $x is false
     */
    var_dump($x);
}
===expect===
