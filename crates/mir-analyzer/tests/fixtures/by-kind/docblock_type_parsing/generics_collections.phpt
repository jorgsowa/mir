===description===
Generic-argument forms: two-arg `array<K, V>`, the one-arg shorthand
(defaults the key to `array-key`), `non-empty-array<K, V>`, `list<T>`,
`non-empty-list<T>`, and a plain user-class generic.
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
class Box {}
/**
 * @template T
 */
class Collection {}

function check_array_generic($x) {
    /**
     * @var array<int, string> $x
     * @mir-check $x is array<int, string>
     */
    var_dump($x);
}

function check_array_generic_one_param($x) {
    /**
     * @var array<string> $x
     * @mir-check $x is array<int|string, string>
     */
    var_dump($x);
}

function check_non_empty_array_generic($x) {
    /**
     * @var non-empty-array<int, string> $x
     * @mir-check $x is non-empty-array<int, string>
     */
    var_dump($x);
}

function check_list_generic($x) {
    /**
     * @var list<int> $x
     * @mir-check $x is list<int>
     */
    var_dump($x);
}

function check_non_empty_list_generic($x) {
    /**
     * @var non-empty-list<int> $x
     * @mir-check $x is non-empty-list<int>
     */
    var_dump($x);
}

function check_user_class_generic($x) {
    /**
     * @var Collection<Box> $x
     * @mir-check $x is Collection<Box>
     */
    var_dump($x);
}
