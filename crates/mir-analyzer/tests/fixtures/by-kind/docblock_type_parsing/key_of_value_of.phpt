===description===
`key-of<T>`/`value-of<T>` statically evaluate the key/value types of an
array shape, generic array, or list, rather than staying an opaque
`TKeyOf`/`TValueOf` placeholder.
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
function check_key_of_array_shape($x) {
    /**
     * @var key-of<array{a: int, b: string}> $x
     * @mir-check $x is 'a'|'b'
     */
    var_dump($x);
}

function check_value_of_array_shape($x) {
    /**
     * @var value-of<array{a: int, b: string}> $x
     * @mir-check $x is int|string
     */
    var_dump($x);
}

function check_key_of_generic_array($x) {
    /**
     * @var key-of<array<int, string>> $x
     * @mir-check $x is int
     */
    var_dump($x);
}

function check_value_of_generic_array($x) {
    /**
     * @var value-of<array<int, string>> $x
     * @mir-check $x is string
     */
    var_dump($x);
}

function check_key_of_list($x) {
    /**
     * @var key-of<list<string>> $x
     * @mir-check $x is int
     */
    var_dump($x);
}
