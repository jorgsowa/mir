===description===
`class-string<T>`/`interface-string<T>` carry a resolved target class, and
`class-string-map<T, V>` expands to a plain array from `class-string` to `V`
(one-arg shorthand defaults `V` to `T` itself).
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
class Foo {}
interface Bar {}

function check_class_string_of($x) {
    /**
     * @var class-string<Foo> $x
     * @mir-check $x is class-string<Foo>
     */
    var_dump($x);
}

function check_interface_string_of($x) {
    /**
     * @var interface-string<Bar> $x
     * @mir-check $x is interface-string<Bar>
     */
    var_dump($x);
}

function check_class_string_map_two_args($x) {
    /**
     * @var class-string-map<Foo, int> $x
     * @mir-check $x is array<class-string, int>
     */
    var_dump($x);
}

function check_class_string_map_one_arg($x) {
    /**
     * @var class-string-map<Foo> $x
     * @mir-check $x is array<class-string, Foo>
     */
    var_dump($x);
}
===expect===
